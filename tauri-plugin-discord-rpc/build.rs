const COMMANDS: &[&str] = &["connect", "disconnect", "set_activity", "clear_activity"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
