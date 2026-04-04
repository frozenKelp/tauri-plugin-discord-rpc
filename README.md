# tauri-plugin-discord-rpc

Discord Rich Presence plugin for Tauri v2. Supports setting activity, timestamps, assets, buttons, and party info.

## Installation

### Rust

Add to your `src-tauri/Cargo.toml`:

```toml
tauri-plugin-discord-rpc = { git = "https://github.com/Youwes09/tauri-plugin-discord-rpc" }
```

### JavaScript

```bash
pnpm add github:Youwes09/tauri-plugin-discord-rpc
# or
npm install github:Youwes09/tauri-plugin-discord-rpc
```

## Setup

Register the plugin in `src-tauri/src/lib.rs`:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_discord_rpc::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

Add the permissions to `src-tauri/capabilities/default.json`.

To allow all commands (recommended):

```json
{
  "permissions": [
    "discord-rpc:default"
  ]
}
```

Or grant individual permissions:

```json
{
  "permissions": [
    "discord-rpc:allow-connect",
    "discord-rpc:allow-disconnect",
    "discord-rpc:allow-set-activity",
    "discord-rpc:allow-clear-activity",
    "discord-rpc:allow-is-running"
  ]
}
```

## Usage

```ts
import {
  connect,
  disconnect,
  setActivity,
  clearActivity,
  isRunning,
} from "tauri-plugin-discord-rpc-api";
```

### Connect

Connect to Discord using your application ID from the [Discord Developer Portal](https://discord.com/developers/applications).

```ts
await connect("your_app_id");
```

### Set Activity

```ts
await setActivity({
  details: "Reading manga",
  state: "Chapter 42 · Reading",
  timestamps: {
    start: Date.now(),
  },
  assets: {
    largeImage: "cover_image_key",
    largeText: "My Manga Title",
    smallImage: "app_logo_key",
    smallText: "My App",
  },
  buttons: [
    { label: "GitHub", url: "https://github.com/yourrepo" },
    { label: "Discord", url: "https://discord.gg/yourinvite" },
  ],
  party: {
    id: "optional_party_id",
    currentSize: 1,
    maxSize: 4,
  },
});
```

### Clear Activity

```ts
await clearActivity();
```

### Check if Discord is Running

```ts
const running = await isRunning(); // boolean
```

### Disconnect

```ts
await disconnect();
```

## API Reference

#### `connect(appId: string): Promise<void>`
Connects to the Discord client using the given application ID.

#### `disconnect(): Promise<void>`
Disconnects from the Discord client and clears any active presence.

#### `setActivity(activity: Activity): Promise<void>`
Sets the current Rich Presence activity. All fields are optional.

```ts
interface Activity {
  details?:    string       // top line of the presence
  state?:      string       // second line
  timestamps?: {
    start?: number          // epoch ms — shows elapsed time
    end?:   number          // epoch ms — shows remaining time
  }
  assets?: {
    largeImage?: string     // art asset key or https:// URL
    largeText?:  string     // tooltip on hover
    smallImage?: string
    smallText?:  string
  }
  buttons?: Array<{
    label: string           // max 32 chars
    url:   string           // must be https://
  }>                        // max 2 buttons
  party?: {
    id?:          string
    currentSize?: number
    maxSize?:     number
  }
}
```

#### `clearActivity(): Promise<void>`
Clears the current Rich Presence activity without disconnecting.

#### `isRunning(): Promise<boolean>`
Returns `true` if Discord is currently running on the system.

## Requirements

- Tauri v2
- Discord desktop app running on the user's machine

## Credits

Developed by [Youwes09](https://github.com/Youwes09), with assistance from [Claude](https://claude.ai) (Anthropic) for core development, debugging, and logic design.
