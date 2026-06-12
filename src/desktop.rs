use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, Emitter};
use tokio::{
  sync::{oneshot, watch, Mutex},
  task::JoinHandle,
  time::sleep,
};
use std::time::Duration;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::models::Activity;
use crate::error::{Error, Result};

const RECONNECT_DELAY: Duration = Duration::from_millis(500);
// Drop-detection latency for an idle connection. Discord's startup is flaky — it can hand out a
// short-lived IPC pipe that dies once it finishes initialising — so this needs to be small enough
// that idle presence reappears promptly, while still being a tiny ping (not the old 500ms activity
// resend). An active write (a presence change) detects a dead pipe immediately on its own.
const LIVENESS:        Duration = Duration::from_secs(2);

#[derive(Clone, Debug)]
enum Cmd {
  Set(Activity),
  Clear,
}

pub struct DiscordRpc<R: Runtime> {
  app:        AppHandle<R>,
  // watch (not a queue): only the latest presence ever matters, so a one-slot channel that
  // overwrites is the right shape — it can never back up, and the worker always sees the newest.
  cmd_tx:     Mutex<Option<watch::Sender<Option<Cmd>>>>,
  running_tx: watch::Sender<bool>,
  running_rx: watch::Receiver<bool>,
  handle:     Mutex<Option<JoinHandle<()>>>,
}

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> Result<DiscordRpc<R>> {
  let (running_tx, running_rx) = watch::channel(false);
  Ok(DiscordRpc {
    app: app.clone(),
    cmd_tx: Mutex::new(None),
    running_tx,
    running_rx,
    handle: Mutex::new(None),
  })
}

impl<R: Runtime> DiscordRpc<R> {
  pub async fn connect(&self, app_id: String) -> Result<()> {
    let mut handle_guard = self.handle.lock().await;

    if handle_guard.as_ref().map(|h| !h.is_finished()).unwrap_or(false) {
      return Err(Error::AlreadyConnected);
    }

    *handle_guard = None;

    let (cmd_tx, cmd_rx)     = watch::channel::<Option<Cmd>>(None);
    let (ready_tx, ready_rx) = oneshot::channel::<Result<()>>();
    let running_tx           = self.running_tx.clone();
    let app                  = self.app.clone();

    // Awaiting the real socket would leave connect() pending forever while Discord is closed, and
    // presence wouldn't appear if Discord is opened after launch. So wire the channel up now and let
    // the worker report just its first attempt, then keep retrying in the background.
    *self.cmd_tx.lock().await = Some(cmd_tx);
    *handle_guard = Some(tokio::spawn(worker(app, app_id, cmd_rx, running_tx, ready_tx)));
    drop(handle_guard);

    ready_rx.await.unwrap_or(Err(Error::NotConnected))
  }

  pub async fn disconnect(&self) -> Result<()> {
    let mut handle_guard = self.handle.lock().await;
    match handle_guard.take() {
      None    => Err(Error::NotConnected),
      Some(h) => {
        // Dropping the sender closes the watch channel, which is how the worker learns to stop.
        *self.cmd_tx.lock().await = None;
        h.abort();
        let _ = self.running_tx.send(false);
        Ok(())
      }
    }
  }

  pub async fn set_activity(&self, payload: Activity) -> Result<()> {
    self.send_cmd(Cmd::Set(payload)).await
  }

  pub async fn clear_activity(&self) -> Result<()> {
    self.send_cmd(Cmd::Clear).await
  }

  pub async fn is_running(&self) -> bool {
    *self.running_rx.borrow()
  }

  async fn send_cmd(&self, cmd: Cmd) -> Result<()> {
    match self.cmd_tx.lock().await.as_ref() {
      // send() only fails if the worker (the sole receiver) is gone — i.e. we're not connected.
      Some(tx) => tx.send(Some(cmd)).map_err(|_| Error::NotConnected),
      None     => Err(Error::NotConnected),
    }
  }
}

// Apply one command on the IPC client. Blocking (the crate is sync), so it runs on a blocking thread
// and hands the client back. Returns whether the write failed so the caller can decide to reconnect.
async fn apply(client: DiscordIpcClient, cmd: Cmd) -> (DiscordIpcClient, bool) {
  tokio::task::spawn_blocking(move || {
    let mut client = client;
    let failed = match &cmd {
      Cmd::Set(act) => client.set_activity(build_activity(act)).is_err(),
      Cmd::Clear    => client.clear_activity().is_err(),
    };
    (client, failed)
  })
  .await
  .expect("spawn_blocking panicked")
}

// Liveness probe: a tiny PING frame (opcode 3) rather than re-broadcasting the whole activity.
// set_activity only ever writes the pipe, so a ping that writes cleanly proves the link is alive —
// at a fraction of the bytes, and without spamming the user's presence.
async fn ping(client: DiscordIpcClient) -> (DiscordIpcClient, bool) {
  tokio::task::spawn_blocking(move || {
    let mut client = client;
    let alive = client.send(serde_json::json!({}), 3).is_ok();
    (client, alive)
  })
  .await
  .expect("spawn_blocking panicked")
}

async fn reconnect_and_restore(
  app_id: &str,
  cmd_rx: &mut watch::Receiver<Option<Cmd>>,
  emit:   &impl Fn(bool),
) -> Option<DiscordIpcClient> {
  let client = match connect_loop(app_id, cmd_rx).await {
    Some(c) => c,
    None    => { emit(false); return None; }
  };
  emit(true);
  // Re-assert the latest desired presence — the watch always holds it. borrow_and_update marks it
  // seen so the main loop's changed() won't immediately re-apply the very same value.
  let latest = cmd_rx.borrow_and_update().clone();
  match latest {
    Some(cmd) => { let (c, _) = apply(client, cmd).await; Some(c) }
    None      => Some(client),
  }
}

async fn worker<R: Runtime>(
  app:        AppHandle<R>,
  app_id:     String,
  mut cmd_rx: watch::Receiver<Option<Cmd>>,
  running_tx: watch::Sender<bool>,
  ready_tx:   oneshot::Sender<Result<()>>,
) {
  let emit = |val: bool| {
    let _ = running_tx.send(val);
    let _ = app.emit("discord-rpc://running", val);
  };

  // One fast initial attempt; looping here would leave connect() pending while Discord is closed.
  let first = {
    let id = app_id.clone();
    tokio::task::spawn_blocking(move || {
      let mut c = DiscordIpcClient::new(&id);
      c.connect().map(|_| c)
    })
    .await
    .ok()
    .and_then(|r| r.ok())
  };

  let mut client = match first {
    Some(c) => {
      emit(true);
      let _ = ready_tx.send(Ok(()));
      c
    }
    None => {
      // Report "not connected" now (don't leave the caller waiting), but keep retrying so opening
      // Discord later attaches presence without a restart.
      emit(false);
      let _ = ready_tx.send(Err(Error::NotConnected));
      match reconnect_and_restore(&app_id, &mut cmd_rx, &emit).await {
        Some(c) => c,
        None    => return,
      }
    }
  };

  loop {
    tokio::select! {
      // Latest presence changed. The watch hands us exactly the newest value — no draining, no
      // backlog, O(1) memory however fast updates arrive.
      changed = cmd_rx.changed() => {
        if changed.is_err() {
          // Every sender dropped (disconnect) → shut down.
          emit(false);
          return;
        }
        let Some(cmd) = cmd_rx.borrow_and_update().clone() else { continue };

        let (c, failed) = apply(client, cmd).await;
        if failed {
          drop(c);
          emit(false);
          match reconnect_and_restore(&app_id, &mut cmd_rx, &emit).await {
            Some(nc) => client = nc,
            None     => return,
          }
        } else {
          client = c;
        }
      }

      // Passive liveness: ping the pipe instead of re-sending presence twice a second, so a drop
      // while idle (e.g. Discord restarted) is noticed and we reconnect and restore.
      _ = sleep(LIVENESS) => {
        let (c, alive) = ping(client).await;
        if !alive {
          drop(c);
          emit(false);
          match reconnect_and_restore(&app_id, &mut cmd_rx, &emit).await {
            Some(nc) => client = nc,
            None     => return,
          }
        } else {
          client = c;
        }
      }
    }
  }
}

async fn connect_loop(
  app_id: &str,
  cmd_rx: &mut watch::Receiver<Option<Cmd>>,
) -> Option<DiscordIpcClient> {
  loop {
    // disconnect() drops the sender; has_changed() then returns Err → stop retrying and shut down.
    if cmd_rx.has_changed().is_err() {
      return None;
    }

    let id = app_id.to_owned();
    match tokio::task::spawn_blocking(move || {
      let mut c = DiscordIpcClient::new(&id);
      c.connect().map(|_| c)
    })
    .await
    {
      Ok(Ok(c)) => return Some(c),
      _ => sleep(RECONNECT_DELAY).await,
    }
  }
}

fn build_activity(payload: &Activity) -> activity::Activity<'_> {
  let mut act = activity::Activity::new();

  if let Some(s) = &payload.state   { act = act.state(s); }
  if let Some(d) = &payload.details { act = act.details(d); }

  if let Some(ts) = &payload.timestamps {
    let mut t = activity::Timestamps::new();
    if let Some(v) = ts.start { t = t.start(v); }
    if let Some(v) = ts.end   { t = t.end(v); }
    act = act.timestamps(t);
  }

  if let Some(assets) = &payload.assets {
    let mut a = activity::Assets::new();
    if let Some(v) = &assets.large_image { a = a.large_image(v); }
    if let Some(v) = &assets.large_text  { a = a.large_text(v); }
    if let Some(v) = &assets.small_image { a = a.small_image(v); }
    if let Some(v) = &assets.small_text  { a = a.small_text(v); }
    act = act.assets(a);
  }

  if let Some(buttons) = &payload.buttons {
    let btns: Vec<activity::Button> = buttons
      .iter()
      .map(|b| activity::Button::new(&b.label, &b.url))
      .collect();
    act = act.buttons(btns);
  }

  if let Some(party) = &payload.party {
    let mut p = activity::Party::new();
    if let Some(id) = &party.id { p = p.id(id); }
    if let (Some(cur), Some(max)) = (party.current_size, party.max_size) {
      p = p.size([cur, max]);
    }
    act = act.party(p);
  }

  act
}
