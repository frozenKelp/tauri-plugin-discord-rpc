# tauri-plugin-discord-rpc

A Tauri v2 plugin for Discord Rich Presence. Handles connection, reconnection, and real-time status without any frontend polling.

## Features

- Connect and disconnect from Discord's IPC socket
- Set and clear Rich Presence with full activity support
- Automatic reconnection when Discord restarts
- Real-time connection state pushed to the frontend via events
- 500ms heartbeat detects dropped connections instantly
- Works on Windows, macOS, and Linux

## Installation

### Rust

Add the plugin to your `src-tauri/Cargo.toml`:

```toml
tauri-plugin-discord-rpc = { git = "https://github.com/Youwes09/tauri-plugin-discord-rpc" }
```

Register it in `src-tauri/src/lib.rs`:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_discord_rpc::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### JavaScript

Install the guest bindings from the repo:

```bash
pnpm add /path/to/tauri-plugin-discord-rpc
```

Add the required permissions to your `capabilities/default.json`:

```json
"discord-rpc:default",
"discord-rpc:allow-connect",
"discord-rpc:allow-disconnect",
"discord-rpc:allow-set-activity",
"discord-rpc:allow-clear-activity",
"discord-rpc:allow-is-running"
```

## Usage

```ts
import { connect, disconnect, setActivity, clearActivity, isRunning } from 'tauri-plugin-discord-rpc-api'
import { listen } from '@tauri-apps/api/event'

// Connect to Discord (blocks until IPC is open or throws)
await connect('YOUR_APP_ID')

// Listen for real-time connection state changes
const unlisten = await listen('discord-rpc://running', ({ payload }) => {
  console.log('Discord connected:', payload) // true | false
})

// Set presence
await setActivity({
  details: 'Building something',
  state: 'In the zone',
  timestamps: { start: Date.now() },
  assets: {
    largeImage: 'my_image_key',
    largeText: 'My App',
  },
  buttons: [
    { label: 'GitHub', url: 'https://github.com' },
  ],
})

// Clear presence
await clearActivity()

// Disconnect
await disconnect()

// Clean up listener
unlisten()
```

## Activity Schema

```ts
interface Activity {
  state?:      string
  details?:    string
  timestamps?: {
    start?: number
    end?:   number
  }
  assets?: {
    largeImage?: string
    largeText?:  string
    smallImage?: string
    smallText?:  string
  }
  buttons?: Array<{
    label: string
    url:   string
  }>
  party?: {
    id?:          string
    currentSize?: number
    maxSize?:     number
  }
}
```

## How It Works

The plugin spawns a background worker that owns the Discord IPC socket. Commands are sent to it via an async channel. If a send fails, it attempts a cheap reconnect before falling back to a full reconnect loop. A 500ms heartbeat pings the socket so dropped connections are detected quickly even when no activity is being set. Connection state changes are emitted as `discord-rpc://running` events the moment they happen — no polling required on the frontend.

## Platform Support

| Platform | Status |
|----------|--------|
| Windows  | ✅     |
| macOS    | ✅     |
| Linux    | ✅     |
| Mobile   | ❌     |

## License

MIT

---

*Code architecture and reconnection methodology developed with assistance from [Claude](https://claude.ai) (Anthropic).*
