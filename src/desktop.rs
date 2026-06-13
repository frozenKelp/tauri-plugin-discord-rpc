use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, Emitter};
use tokio::{
  sync::{oneshot, watch, Mutex},
  task::JoinHandle,
  time::sleep,
};
use std::time::Duration;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::models::{Activity, User};
use crate::error::{Error, Result};

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

// Result of reading Discord's response to one write.
#[derive(Debug, PartialEq)]
enum Outcome {
  Ok,                // ack or PONG — link healthy
  Rejected(String),  // Discord refused the activity; link stays up
  Dead,              // CLOSE frame or read failure — reconnect
}

// Centralises all webview event emission + shared state updates for the worker.
struct Events<'a, R: Runtime> {
  app:        &'a AppHandle<R>,
  running_tx: &'a watch::Sender<bool>,
  user_tx:    &'a watch::Sender<Option<User>>,
}

impl<R: Runtime> Events<'_, R> {
  fn connected(&self, val: bool) {
    let _ = self.running_tx.send(val);
    let _ = self.app.emit("discord-rpc://connected", val);
    if !val {
      let _ = self.user_tx.send(None);
    }
  }
  fn ready(&self, user: &User) {
    let _ = self.user_tx.send(Some(user.clone()));
    let _ = self.app.emit("discord-rpc://ready", user.clone());
  }
  fn error(&self, msg: &str) {
    let _ = self.app.emit("discord-rpc://error", msg.to_string());
  }
}

pub struct DiscordRpc<R: Runtime> {
  app:        AppHandle<R>,
  // watch (not a queue): only the latest presence ever matters, so a one-slot channel that
  // overwrites is the right shape — it can never back up, and the worker always sees the newest.
  cmd_tx:     Mutex<Option<watch::Sender<Option<Cmd>>>>,
  running_tx: watch::Sender<bool>,
  running_rx: watch::Receiver<bool>,
  user_tx:    watch::Sender<Option<User>>,
  user_rx:    watch::Receiver<Option<User>>,
  handle:     Mutex<Option<JoinHandle<()>>>,
}

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> Result<DiscordRpc<R>> {
  let (running_tx, running_rx) = watch::channel(false);
  let (user_tx, user_rx)       = watch::channel(None);
  Ok(DiscordRpc {
    app: app.clone(),
    cmd_tx: Mutex::new(None),
    running_tx,
    running_rx,
    user_tx,
    user_rx,
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
    let user_tx              = self.user_tx.clone();
    let app                  = self.app.clone();

    // Awaiting the real socket would leave connect() pending forever while Discord is closed, and
    // presence wouldn't appear if Discord is opened after launch. So wire the channel up now and let
    // the worker report just its first attempt, then keep retrying in the background.
    *self.cmd_tx.lock().await = Some(cmd_tx);
    *handle_guard = Some(tokio::spawn(worker(app, app_id, cmd_rx, running_tx, user_tx, ready_tx)));
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
        let _ = self.user_tx.send(None);
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

  pub async fn is_connected(&self) -> bool {
    *self.running_rx.borrow()
  }

  pub async fn get_current_user(&self) -> Option<User> {
    self.user_rx.borrow().clone()
  }

  async fn send_cmd(&self, cmd: Cmd) -> Result<()> {
    match self.cmd_tx.lock().await.as_ref() {
      // send() only fails if the worker (the sole receiver) is gone — i.e. we're not connected.
      Some(tx) => tx.send(Some(cmd)).map_err(|_| Error::NotConnected),
      None     => Err(Error::NotConnected),
    }
  }
}

// Apply one command, then READ Discord's echoed response (lock-step). Blocking → blocking thread.
async fn apply(client: DiscordIpcClient, cmd: Cmd) -> (DiscordIpcClient, Outcome) {
  tokio::task::spawn_blocking(move || {
    let mut client = client;
    let wrote = match &cmd {
      Cmd::Set(act) => client.set_activity(build_activity(act)),
      Cmd::Clear    => client.clear_activity(),
    };
    let outcome = match wrote {
      Err(_) => Outcome::Dead,
      Ok(()) => match client.recv() {
        Err(_)      => Outcome::Dead,
        Ok((op, v)) => classify_frame(op, &v),
      },
    };
    (client, outcome)
  })
  .await
  .expect("spawn_blocking panicked")
}

// Liveness PING (opcode 3) — now READS the PONG back (no more leaked responses in the buffer).
// A clean write + read proves the link is alive, at a fraction of the bytes of a presence resend.
async fn ping(client: DiscordIpcClient) -> (DiscordIpcClient, Outcome) {
  tokio::task::spawn_blocking(move || {
    let mut client = client;
    let outcome = match client.send(serde_json::json!({}), 3) {
      Err(_) => Outcome::Dead,
      Ok(()) => match client.recv() {
        Err(_)      => Outcome::Dead,
        Ok((op, v)) => classify_frame(op, &v),
      },
    };
    (client, outcome)
  })
  .await
  .expect("spawn_blocking panicked")
}

async fn reconnect_and_restore<R: Runtime>(
  app_id: &str,
  cmd_rx: &mut watch::Receiver<Option<Cmd>>,
  ev:     &Events<'_, R>,
) -> Option<DiscordIpcClient> {
  let (client, user) = match connect_loop(app_id, cmd_rx).await {
    Some(pair) => pair,
    None       => { ev.connected(false); return None; }
  };
  ev.connected(true);
  ev.ready(&user);

  // Re-assert the latest desired presence — the watch always holds it. borrow_and_update marks it
  // seen so the main loop's changed() won't immediately re-apply the very same value.
  let latest = cmd_rx.borrow_and_update().clone();
  match latest {
    Some(cmd) => {
      let (c, outcome) = apply(client, cmd).await;
      if let Outcome::Rejected(msg) = &outcome {
        ev.error(msg);
      }
      Some(c) // if Dead, the main loop's next ping reconnects again
    }
    None => Some(client),
  }
}

async fn worker<R: Runtime>(
  app:        AppHandle<R>,
  app_id:     String,
  mut cmd_rx: watch::Receiver<Option<Cmd>>,
  running_tx: watch::Sender<bool>,
  user_tx:    watch::Sender<Option<User>>,
  ready_tx:   oneshot::Sender<Result<()>>,
) {
  let ev = Events { app: &app, running_tx: &running_tx, user_tx: &user_tx };

  // One fast initial attempt; looping here would leave connect() pending while Discord is closed.
  let first = {
    let id = app_id.clone();
    tokio::task::spawn_blocking(move || handshake(&id)).await.ok()
  };

  let mut client = match first {
    Some(Ok((c, user))) => {
      ev.connected(true);
      ev.ready(&user);
      let _ = ready_tx.send(Ok(()));
      c
    }
    other => {
      // Preserve a real protocol error (e.g. invalid client_id) for the connect() caller, else
      // report NotConnected. Then keep retrying so opening Discord later attaches presence.
      let err = match other {
        Some(Err(e)) => e,
        _            => Error::NotConnected,
      };
      ev.connected(false);
      let _ = ready_tx.send(Err(err));
      match reconnect_and_restore(&app_id, &mut cmd_rx, &ev).await {
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
          ev.connected(false);
          return;
        }
        let Some(cmd) = cmd_rx.borrow_and_update().clone() else { continue };

        let (c, outcome) = apply(client, cmd).await;
        match outcome {
          Outcome::Ok            => client = c,
          Outcome::Rejected(msg) => { ev.error(&msg); client = c; } // stay connected
          Outcome::Dead          => {
            drop(c);
            ev.connected(false);
            match reconnect_and_restore(&app_id, &mut cmd_rx, &ev).await {
              Some(nc) => client = nc,
              None     => return,
            }
          }
        }
      }

      // Passive liveness: ping the pipe instead of re-sending presence twice a second, so a drop
      // while idle (e.g. Discord restarted) is noticed and we reconnect and restore.
      _ = sleep(LIVENESS) => {
        let (c, outcome) = ping(client).await;
        if outcome == Outcome::Dead {
          drop(c);
          ev.connected(false);
          match reconnect_and_restore(&app_id, &mut cmd_rx, &ev).await {
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
) -> Option<(DiscordIpcClient, User)> {
  let mut attempt = 0u32;
  loop {
    // disconnect() drops the sender; has_changed() then returns Err → stop retrying and shut down.
    if cmd_rx.has_changed().is_err() {
      return None;
    }

    let id = app_id.to_owned();
    match tokio::task::spawn_blocking(move || handshake(&id)).await {
      Ok(Ok(pair)) => return Some(pair),
      _ => {
        sleep(backoff_delay(attempt)).await;
        attempt = attempt.saturating_add(1);
      }
    }
  }
}

fn build_activity(payload: &Activity) -> activity::Activity<'_> {
  let mut act = activity::Activity::new();

  if let Some(s) = &payload.state       { act = act.state(s); }
  if let Some(u) = &payload.state_url   { act = act.state_url(u); }
  if let Some(d) = &payload.details     { act = act.details(d); }
  if let Some(u) = &payload.details_url { act = act.details_url(u); }

  if let Some(t) = payload.activity_type {
    act = act.activity_type(match t {
      2 => activity::ActivityType::Listening,
      3 => activity::ActivityType::Watching,
      5 => activity::ActivityType::Competing,
      _ => activity::ActivityType::Playing,
    });
  }
  if let Some(d) = payload.status_display_type {
    act = act.status_display_type(match d {
      1 => activity::StatusDisplayType::State,
      2 => activity::StatusDisplayType::Details,
      _ => activity::StatusDisplayType::Name,
    });
  }

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
    let btns: Vec<activity::Button> = capped_buttons(buttons)
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

// Discord allows at most 2 buttons; silently take the first two.
fn capped_buttons(buttons: &[crate::models::Button]) -> &[crate::models::Button] {
  &buttons[..buttons.len().min(2)]
}

// Capped exponential reconnect backoff: 500ms, 1s, 2s, 4s, 8s, then held at 8s.
fn backoff_delay(attempt: u32) -> Duration {
  let ms = 500u64.saturating_mul(1u64 << attempt.min(4));
  Duration::from_millis(ms.min(8000))
}

// Pure classifier for a received (opcode, json) frame. Read failures are mapped to Outcome::Dead at
// the call site, so this stays free of the crate's error type and is testable without Discord.
fn classify_frame(opcode: u32, data: &serde_json::Value) -> Outcome {
  if opcode == 2 {
    return Outcome::Dead; // CLOSE
  }
  if data.get("evt").and_then(serde_json::Value::as_str) == Some("ERROR") {
    let msg = data
      .get("data")
      .and_then(|d| d.get("message"))
      .and_then(serde_json::Value::as_str)
      .unwrap_or("activity rejected")
      .to_string();
    return Outcome::Rejected(msg);
  }
  Outcome::Ok
}

// Parse a handshake response frame: READY -> User, or an ERROR frame -> Err(Rpc(message)). Discord's
// user is snake_case, so parse the Value by hand (our User serializes camelCase for the JS wire).
fn parse_ready(frame: &serde_json::Value) -> Result<User> {
  if frame.get("evt").and_then(serde_json::Value::as_str) == Some("ERROR") {
    let msg = frame
      .get("data")
      .and_then(|d| d.get("message"))
      .and_then(serde_json::Value::as_str)
      .unwrap_or("handshake rejected")
      .to_string();
    return Err(Error::Rpc(msg));
  }
  let u = frame
    .get("data")
    .and_then(|d| d.get("user"))
    .ok_or_else(|| Error::Rpc("READY frame missing user".to_string()))?;

  let s = |k: &str| u.get(k).and_then(serde_json::Value::as_str).map(str::to_string);
  Ok(User {
    id: s("id").unwrap_or_default(),
    username: s("username").unwrap_or_default(),
    discriminator: s("discriminator"),
    global_name: s("global_name"),
    avatar: s("avatar"),
  })
}

// Hand-rolled handshake so we can KEEP the READY frame (the crate's connect() throws it away).
// Transport failures map to NotConnected (Discord closed, retryable); an explicit ERROR frame maps
// to Rpc (e.g. invalid client_id) via parse_ready.
fn handshake(app_id: &str) -> Result<(DiscordIpcClient, User)> {
  let mut client = DiscordIpcClient::new(app_id);
  client.connect_ipc().map_err(|_| Error::NotConnected)?;
  client
    .send(serde_json::json!({ "v": 1, "client_id": app_id }), 0)
    .map_err(|_| Error::NotConnected)?;
  let (_op, frame) = client.recv().map_err(|_| Error::NotConnected)?;
  let user = parse_ready(&frame)?;
  Ok((client, user))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn backoff_is_capped_exponential() {
    assert_eq!(backoff_delay(0), Duration::from_millis(500));
    assert_eq!(backoff_delay(1), Duration::from_millis(1000));
    assert_eq!(backoff_delay(2), Duration::from_millis(2000));
    assert_eq!(backoff_delay(3), Duration::from_millis(4000));
    assert_eq!(backoff_delay(4), Duration::from_millis(8000));
    assert_eq!(backoff_delay(9), Duration::from_millis(8000)); // capped
  }

  #[test]
  fn classify_close_opcode_is_dead() {
    assert_eq!(classify_frame(2, &serde_json::json!({})), Outcome::Dead);
  }

  #[test]
  fn classify_error_evt_is_rejected_with_message() {
    let v = serde_json::json!({ "evt": "ERROR", "data": { "message": "Invalid asset" } });
    assert_eq!(classify_frame(1, &v), Outcome::Rejected("Invalid asset".to_string()));
  }

  #[test]
  fn classify_normal_frame_is_ok() {
    let v = serde_json::json!({ "cmd": "SET_ACTIVITY", "data": {} });
    assert_eq!(classify_frame(1, &v), Outcome::Ok);
    assert_eq!(classify_frame(4, &serde_json::json!({})), Outcome::Ok); // PONG
  }

  #[test]
  fn parse_ready_extracts_user_from_snake_case() {
    let frame = serde_json::json!({
      "cmd": "DISPATCH", "evt": "READY",
      "data": { "v": 1, "user": {
        "id": "42", "username": "bob",
        "discriminator": "0", "global_name": "Bob", "avatar": "abc"
      }}
    });
    let u = parse_ready(&frame).unwrap();
    assert_eq!(u.id, "42");
    assert_eq!(u.username, "bob");
    assert_eq!(u.global_name.as_deref(), Some("Bob"));
  }

  #[test]
  fn parse_ready_error_frame_is_err() {
    let frame = serde_json::json!({ "evt": "ERROR", "data": { "message": "Invalid client ID" } });
    let err = parse_ready(&frame).unwrap_err();
    assert!(matches!(err, Error::Rpc(m) if m == "Invalid client ID"));
  }

  #[test]
  fn capped_buttons_limits_to_two() {
    let btns = vec![
      crate::models::Button { label: "a".into(), url: "u".into() },
      crate::models::Button { label: "b".into(), url: "u".into() },
      crate::models::Button { label: "c".into(), url: "u".into() },
    ];
    assert_eq!(capped_buttons(&btns).len(), 2);
    assert_eq!(capped_buttons(&btns[..1]).len(), 1);
  }
}
