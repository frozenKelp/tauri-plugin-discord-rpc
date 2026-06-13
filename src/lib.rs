use tauri::{plugin::{Builder, TauriPlugin}, Manager, Runtime};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::DiscordRpc;
#[cfg(mobile)]
use mobile::DiscordRpc;

pub trait DiscordRpcExt<R: Runtime> {
  fn discord_rpc(&self) -> &DiscordRpc<R>;
}

impl<R: Runtime, T: Manager<R>> DiscordRpcExt<R> for T {
  fn discord_rpc(&self) -> &DiscordRpc<R> {
    self.state::<DiscordRpc<R>>().inner()
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("discord-rpc")
    .invoke_handler(tauri::generate_handler![
      commands::connect,
      commands::disconnect,
      commands::set_activity,
      commands::clear_activity,
      commands::is_connected,
      commands::get_current_user,
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let discord_rpc = mobile::init(app, api)?;
      #[cfg(desktop)]
      let discord_rpc = desktop::init(app, api)?;
      app.manage(discord_rpc);
      Ok(())
    })
    .build()
}
