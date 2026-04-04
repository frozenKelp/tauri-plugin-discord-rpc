# tauri-plugin-discord-rpc

A Tauri v2 plugin for Discord Rich Presence. Wraps Discord's IPC socket with a clean async API, automatic reconnection, and real-time connection state pushed to the frontend.

## Features

- Connect, disconnect, and set/clear Rich Presence from any Tauri app
- Automatic reconnection when Discord restarts — no manual polling required
- Real-time connection status via Tauri events (no frontend polling)
- Heartbeat ping detects dropped connections within 500ms
- Full activity support: state, details, timestamps, assets, buttons, party

## Installation

Add the plugin to your `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-discord-rpc = { path = "tauri-plugin-discord-rpc" }
```

Register it in your Tauri app:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_discord_rpc::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

Install the JS bindings:

```bash
npm install tauri-plugin-discord-rpc-api
```

## Usage

```js
import {
  connect,
  disconnect,
  setActivity,
  clearActivity,
  isRunning,
} from 'tauri-plugin-discord-rpc-api'
import { listen } from '@tauri-apps/api/event'

// Connect to Discord
await connect('YOUR_APP_ID')

// Listen for connection state changes (instant, no polling)
const unlisten = await listen('discord-rpc://running', ({ payload }) => {
  console.log('Discord connected:', payload)
})

// Set presence
await setActivity({
  details: 'Building something',
  state: 'In the zone',
  timestamps: { start: Date.now() },
  buttons: [{ label: 'GitHub', url: 'https://github.com' }],
})

// Clear presence
await clearActivity()

// Disconnect
await disconnect()
```

## Activity Schema

```ts
interface Activity {
  state?:      string
  details?:    string
  timestamps?: { start?: number; end?: number }
  assets?: {
    largeImage?: string
    largeText?:  string
    smallImage?: string
    smallText?:  string
  }
  buttons?: Array<{ label: string; url: string }>
  party?: {
    id?:          string
    currentSize?: number
    maxSize?:     number
  }
}
```

## How It Works

The plugin spawns a background worker that owns the Discord IPC socket. Commands (`setActivity`, `clearActivity`) are sent to the worker via an async channel. If a send fails, the worker attempts a cheap reconnect on the same thread before falling back to a full reconnect loop. A 500ms heartbeat pings the socket continuously so dropped connections are caught quickly regardless of activity. Connection state changes are emitted as `discord-rpc://running` events the moment they happen.

## Platform Support

| Platform | Status  |
|----------|---------|
| Windows  | ✅      |
| macOS    | ✅      |
| Linux    | ✅      |
| Mobile   | ❌      |

## License

MIT

---

*Code architecture and reconnection methodology developed with assistance from [Claude](https://claude.ai) (Anthropic).*
