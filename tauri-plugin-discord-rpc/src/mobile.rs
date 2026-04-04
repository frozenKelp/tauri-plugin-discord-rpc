use serde::de::DeserializeOwned;
use tauri::{plugin::{PluginApi, PluginHandle}, AppHandle, Runtime};

pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<DiscordRpc<R>> {
  unimplemented!("discord-rpc is not supported on mobile")
}

pub struct DiscordRpc<R: Runtime>(std::marker::PhantomData<R>);

impl<R: Runtime> DiscordRpc<R> {
  pub async fn connect(&self, _app_id: String) -> crate::Result<()> { unimplemented!() }
  pub async fn disconnect(&self) -> crate::Result<()> { unimplemented!() }
  pub async fn set_activity(&self, _payload: crate::models::Activity) -> crate::Result<()> { unimplemented!() }
  pub async fn clear_activity(&self) -> crate::Result<()> { unimplemented!() }
  pub async fn is_running(&self) -> bool { false }
}