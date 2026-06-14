<script>
  import { connect, disconnect, isConnected, getCurrentUser } from 'tauri-plugin-discord-rpc-api'
  import { listen } from '@tauri-apps/api/event'
  import { onMount, onDestroy } from 'svelte'
  import { DISCORD_APP_ID } from './custom-payloads.js'

  let { onConnectedChange = () => {} } = $props()

  let appId = $state(DISCORD_APP_ID)
  let connected = $state(false)
  let busy = $state(false)
  let user = $state(null)
  let msg = $state('')

  const unlisteners = []

  function setConnected(v) {
    connected = v
    onConnectedChange(v)
    if (!v) user = null
  }

  onMount(async () => {
    connected = await isConnected().catch(() => false)
    if (connected) { user = await getCurrentUser().catch(() => null); onConnectedChange(true) }
    unlisteners.push(await listen('discord-rpc://connected', ({ payload }) => setConnected(payload)))
    unlisteners.push(await listen('discord-rpc://ready', ({ payload }) => { user = payload }))
  })
  onDestroy(() => unlisteners.forEach(u => u?.()))

  async function handleConnect() {
    busy = true; msg = ''
    // connect() resolves Ok once the first attempt succeeds; reflect it immediately rather than
    // waiting on the connected event (and it keeps retrying in the background on its own).
    try { await connect(appId); setConnected(true) }
    catch (e) { msg = String(e); setConnected(false) }
    finally { busy = false }
  }
  async function handleDisconnect() {
    busy = true; msg = ''
    // disconnect() tears down the worker without emitting a `connected:false` event, so update local state directly
    try { await disconnect() } catch (e) { msg = String(e) }
    setConnected(false)
    busy = false
  }
</script>

<section class="connbar">
  <input class="appid" bind:value={appId} placeholder="Discord app. id" disabled={connected} />
  {#if connected}
    <button type="button" onclick={handleDisconnect} disabled={busy}>Disconnect</button>
  {:else}
    <button type="button" onclick={handleConnect} disabled={busy}>Connect</button>
  {/if}
  <span class="status"><span class="dot" class:on={connected}></span>{connected ? 'connected' : 'disconnected'}</span>
  {#if user}<span class="user">as <b>{user.globalName ?? user.username}</b></span>{/if}
  {#if msg}<span class="msg">{msg}</span>{/if}
</section>
