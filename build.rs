const COMMANDS: &[&str] = &[
  "connect",
  "disconnect",
  "set_activity",
  "clear_activity",
  "is_connected",
  "get_current_user",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
