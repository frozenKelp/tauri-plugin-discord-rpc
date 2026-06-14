const COMMANDS: &[&str] = &[
  "connect",
  "disconnect",
  "set_activity",
  "set_activity_raw",
  "clear_activity",
  "is_connected",
  "get_current_user",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
