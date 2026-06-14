use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::error::Error;
use crate::models::{Activity, User};

pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<DiscordRpc<R>> {
  // RPC over IPC is desktop-only; return a stub instead of panicking the whole app.
  Ok(DiscordRpc(std::marker::PhantomData))
}

pub struct DiscordRpc<R: Runtime>(std::marker::PhantomData<R>);

impl<R: Runtime> DiscordRpc<R> {
  pub async fn connect(&self, _app_id: String) -> crate::Result<()> { Err(Error::Unsupported) }
  pub async fn disconnect(&self) -> crate::Result<()> { Err(Error::Unsupported) }
  pub async fn set_activity(&self, _payload: Activity) -> crate::Result<()> { Err(Error::Unsupported) }
  pub async fn set_activity_raw(&self, _payload: serde_json::Value) -> crate::Result<()> { Err(Error::Unsupported) }
  pub async fn clear_activity(&self) -> crate::Result<()> { Err(Error::Unsupported) }
  pub async fn is_connected(&self) -> bool { false }
  pub async fn get_current_user(&self) -> Option<User> { None }
}
