use tauri::{command, AppHandle, Runtime};
use crate::{models::Activity, Result, DiscordRpcExt};

#[command]
pub(crate) async fn connect<R: Runtime>(app: AppHandle<R>, app_id: String) -> Result<()> {
  app.discord_rpc().connect(app_id).await
}

#[command]
pub(crate) async fn disconnect<R: Runtime>(app: AppHandle<R>) -> Result<()> {
  app.discord_rpc().disconnect().await
}

#[command]
pub(crate) async fn set_activity<R: Runtime>(app: AppHandle<R>, payload: Activity) -> Result<()> {
  app.discord_rpc().set_activity(payload).await
}

#[command]
pub(crate) async fn clear_activity<R: Runtime>(app: AppHandle<R>) -> Result<()> {
  app.discord_rpc().clear_activity().await
}

#[command]
pub(crate) async fn is_running<R: Runtime>(app: AppHandle<R>) -> bool {
  app.discord_rpc().is_running().await
}
