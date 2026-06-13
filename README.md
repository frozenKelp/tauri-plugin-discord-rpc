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
    "discord-rpc:allow-is-connected",
    "discord-rpc:allow-get-current-user"
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
  isConnected,
  getCurrentUser,
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

### Check if Connected

```ts
const connected = await isConnected(); // boolean — is the RPC link to Discord live?
```

### Get the Logged-in User

```ts
const user = await getCurrentUser(); // User | null
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
  name?:       string       // activity name; substitute app's registered name
  details?:    string       // top line of the presence
  state?:      string       // second line
  detailsUrl?: string       // makes the details line a clickable link
  stateUrl?:   string       // makes the state line a clickable link
  activityType?:      number // 0 Playing, 2 Listening, 3 Watching, 5 Competing
  statusDisplayType?: number // 0 Name, 1 State, 2 Details // which line shows as the headline
  timestamps?: {
    start?: number          // epoch ms — shows elapsed time
    end?:   number          // epoch ms — shows remaining time
  }
  assets?: {
    largeImage?: string     // art asset key or https:// URL
    largeText?:  string     // tooltip on hover
    largeUrl?:   string     // large image , clickable link
    smallImage?: string
    smallText?:  string
    smallUrl?:   string     // small image , clickable link
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

#### `isConnected(): Promise<boolean>`
Returns `true` while the RPC connection to Discord is live (not whether a Discord process
merely exists on the system).

#### `getCurrentUser(): Promise<User | null>`
Returns the logged-in Discord user captured from the `READY` handshake, or `null` if not
connected.

```ts
interface User {
  id:             string
  username:       string
  discriminator?: string
  globalName?:    string
  avatar?:        string   // avatar hash
}
```

### Events

The plugin emits these events (listen via `@tauri-apps/api/event`):

| Event | Payload | When |
|-------|---------|------|
| `discord-rpc://connected` | `boolean` | Connection comes up (`true`) or drops (`false`). |
| `discord-rpc://ready`     | `User`    | Right after a successful handshake — carries the logged-in user. |
| `discord-rpc://error`     | `string`  | Discord rejected a presence update; payload is its message. The connection stays up. |

> **Note on the activity name:** you may substitute the registered application's name with `name`.

## Migration

- `isRunning()` → **`isConnected()`** (same return type; clearer meaning).
- Permission `discord-rpc:allow-is-running` → **`discord-rpc:allow-is-connected`**, and add
  `discord-rpc:allow-get-current-user` if you grant permissions individually.
- New, additive: the `discord-rpc://ready` / `discord-rpc://error` events and
  `getCurrentUser()`. `setActivity()` keeps its resolve-on-accept contract; watch the
  `error` event to learn when Discord rejects an update.

## Requirements

- Tauri v2
- Discord desktop app running on the user's machine

## Credits

Developed by [Youwes09](https://github.com/Youwes09), with assistance from [Claude](https://claude.ai) (Anthropic) for core development, debugging, and logic design.
