use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, Emitter};
use tokio::{
  sync::{mpsc, oneshot, watch, Mutex},
  task::JoinHandle,
  time::sleep,
};
use std::time::Duration;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::models::Activity;
use crate::error::{Error, Result};

const RECONNECT_DELAY: Duration = Duration::from_millis(500);
const HEARTBEAT:       Duration = Duration::from_millis(500);

#[derive(Clone, Debug)]
enum Cmd {
  Set(Activity),
  Clear,
}

pub struct DiscordRpc<R: Runtime> {
  app:        AppHandle<R>,
  cmd_tx:     Mutex<Option<mpsc::UnboundedSender<Cmd>>>,
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

    let (cmd_tx, cmd_rx)     = mpsc::unbounded_channel::<Cmd>();
    let (ready_tx, ready_rx) = oneshot::channel::<Result<()>>();
    let running_tx           = self.running_tx.clone();
    let app                  = self.app.clone();

    // Awaiting the real socket would leave connect() pending forever while Discord is closed,
    // if Discord is opened after launch presence wouldnt show.
    // keep the worker retrying in the background, 
    // wire the command channel up now so activity sent before it connects isn't lost.
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
      Some(tx) => tx.send(cmd).map_err(|_| Error::NotConnected),
      None     => Err(Error::NotConnected),
    }
  }
}

async fn reconnect_and_restore(
  app_id:   &str,
  cmd_rx:   &mut mpsc::UnboundedReceiver<Cmd>,
  last_cmd: &mut Option<Cmd>,
  emit:     &impl Fn(bool),
) -> Option<DiscordIpcClient> {
  match connect_loop(app_id, cmd_rx, last_cmd).await {
    None => {
      emit(false);
      None
    }
    Some(mut c) => {
      emit(true);
      if let Some(lc) = last_cmd.as_ref() {
        let (nc, _) = tokio::task::spawn_blocking({
          let lc = lc.clone();
          move || {
            let r = match &lc {
              Cmd::Set(act) => c.set_activity(build_activity(act)),
              Cmd::Clear    => c.clear_activity(),
            };
            (c, r)
          }
        })
        .await
        .expect("spawn_blocking panicked");
        Some(nc)
      } else {
        Some(c)
      }
    }
  }
}

async fn worker<R: Runtime>(
  app:        AppHandle<R>,
  app_id:     String,
  mut cmd_rx: mpsc::UnboundedReceiver<Cmd>,
  running_tx: watch::Sender<bool>,
  ready_tx:   oneshot::Sender<Result<()>>,
) {
  let emit = |val: bool| {
    let _ = running_tx.send(val);
    let _ = app.emit("discord-rpc://running", val);
  };

  // Report first attempt , looping here leaves connect() pending.
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

  let mut last_cmd: Option<Cmd> = None;

  let mut client = match first {
    Some(c) => {
      emit(true);
      let _ = ready_tx.send(Ok(()));
      c
    }
    None => {
      // keep retrying so opening Discord later attaches presence without a restart.
      // forward "not connected", so it isn't left waiting.
      emit(false);
      let _ = ready_tx.send(Err(Error::NotConnected));
      // Reuse the reconnect path: it collapses the backlog and shows the latest presence on connect.
      match reconnect_and_restore(&app_id, &mut cmd_rx, &mut last_cmd, &emit).await {
        Some(c) => c,
        None    => return,
      }
    }
  };

  loop {
    tokio::select! {
      cmd = cmd_rx.recv() => {
        let cmd = match cmd {
          Some(c) => c,
          None => {
            emit(false);
            return;
          }
        };

        let cmd = drain_to_latest(&mut cmd_rx, cmd);
        last_cmd = Some(cmd.clone());

        enum Outcome {
          Ok(DiscordIpcClient),
          CheapReconnectOk(DiscordIpcClient),
          NeedsFullReconnect(DiscordIpcClient),
        }

        let last_for_blocking = last_cmd.clone();
        let outcome = tokio::task::spawn_blocking(move || {
          let failed = match &cmd {
            Cmd::Set(act) => client.set_activity(build_activity(act)).is_err(),
            Cmd::Clear    => client.clear_activity().is_err(),
          };

          if failed {
            if client.reconnect().is_ok() {
              if let Some(lc) = last_for_blocking {
                let _ = match &lc {
                  Cmd::Set(act) => client.set_activity(build_activity(act)),
                  Cmd::Clear    => client.clear_activity(),
                };
              }
              Outcome::CheapReconnectOk(client)
            } else {
              Outcome::NeedsFullReconnect(client)
            }
          } else {
            Outcome::Ok(client)
          }
        })
        .await
        .expect("spawn_blocking panicked");

        match outcome {
          Outcome::Ok(c) | Outcome::CheapReconnectOk(c) => {
            client = c;
          }
          Outcome::NeedsFullReconnect(c) => {
            drop(c);
            emit(false);
            match reconnect_and_restore(&app_id, &mut cmd_rx, &mut last_cmd, &emit).await {
              Some(c) => client = c,
              None    => return,
            }
          }
        }
      }

      _ = sleep(HEARTBEAT) => {
        let lc = last_cmd.clone();
        let (c, alive) = tokio::task::spawn_blocking(move || {
          let ok = match &lc {
            Some(Cmd::Set(act)) => client.set_activity(build_activity(act)).is_ok(),
            Some(Cmd::Clear) | None => client.clear_activity().is_ok(),
          };
          (client, ok)
        })
        .await
        .expect("spawn_blocking panicked");

        client = c;

        if !alive {
          emit(false);
          match reconnect_and_restore(&app_id, &mut cmd_rx, &mut last_cmd, &emit).await {
            Some(c) => client = c,
            None    => return,
          }
        }
      }
    }
  }
}

async fn connect_loop(
  app_id:   &str,
  cmd_rx:   &mut mpsc::UnboundedReceiver<Cmd>,
  last_cmd: &mut Option<Cmd>,
) -> Option<DiscordIpcClient> {
  loop {
    // Shutdown is signalled by disconnect() dropping the command sender, which closes the channel.
    if cmd_rx.is_closed() {
      return None;
    }

    // Only the most recent presence matters, so collapse anything queued while we're down to just
    // the latest — otherwise the unbounded channel grows for the whole disconnected period (e.g.
    // reading with Discord closed). The latest is applied on connect. We must NOT exit on a command
    // here — an earlier version did, killing the worker mid-reconnect; we only remember it.
    while let Ok(c) = cmd_rx.try_recv() {
      *last_cmd = Some(c);
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

fn drain_to_latest(rx: &mut mpsc::UnboundedReceiver<Cmd>, first: Cmd) -> Cmd {
  let mut latest = first;
  while let Ok(next) = rx.try_recv() {
    latest = next;
  }
  latest
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