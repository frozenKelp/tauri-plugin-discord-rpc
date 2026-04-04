# tauri-plugin-discord-rpc

Discord Rich Presence plugin for Tauri v2.

## Installation

### Rust

Add to your `src-tauri/Cargo.toml`:

```toml
tauri-plugin-discord-rpc = { git = "https://github.com/Youwes09/tauri-plugin-discord-rpc" }
```

### JavaScript

```bash
pnpm add github:Youwes09/tauri-plugin-discord-rpc
```

## Setup

Register the plugin in `src-tauri/src/lib.rs`:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_discord_rpc::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

Add the permission to `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "discord-rpc:default"
  ]
}
```

## Usage

```ts
import { connect, disconnect, setActivity, clearActivity } from "tauri-plugin-discord-rpc";

// Connect to Discord
await connect();

// Set activity
await setActivity({
  state: "Browsing",
  details: "Reading manga",
  largeImageKey: "your-image-key",
});

// Clear activity
await clearActivity();

// Disconnect
await disconnect();
```

## Requirements

- Tauri v2
- Discord desktop app running on the user's machine
