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
  let log = $state([])

  const unlisteners = []

  function addLog(kind, detail) {
    log = [{ t: new Date().toLocaleTimeString(), kind, detail }, ...log].slice(0, 50)
  }
  function setConnected(v) {
    connected = v
    onConnectedChange(v)
  }

  onMount(async () => {
    connected = await isConnected().catch(() => false)
    if (connected) { user = await getCurrentUser().catch(() => null); onConnectedChange(true) }
    unlisteners.push(await listen('discord-rpc://connected', ({ payload }) => {
      setConnected(payload); addLog('connected', String(payload))
      if (!payload) user = null
    }))
    unlisteners.push(await listen('discord-rpc://ready', ({ payload }) => {
      user = payload; addLog('ready', payload?.username ?? '')
    }))
    unlisteners.push(await listen('discord-rpc://error', ({ payload }) => {
      addLog('error', String(payload))
    }))
  })
  onDestroy(() => unlisteners.forEach(u => u?.()))

  async function handleConnect() {
    busy = true
    try { await connect(appId); addLog('action', 'connect ok') }
    catch (e) { addLog('error', String(e)) }
    finally { busy = false }
  }
  async function handleDisconnect() {
    busy = true
    try { await disconnect() } catch (e) { addLog('error', String(e)) }
    finally { busy = false; user = null }
  }
</script>

<section class="bar">
  <div class="row">
    <input bind:value={appId} placeholder="Discord application id" disabled={connected} />
    {#if connected}
      <button onclick={handleDisconnect} disabled={busy}>Disconnect</button>
    {:else}
      <button onclick={handleConnect} disabled={busy}>Connect</button>
    {/if}
    <span class="dot" class:on={connected}></span>
    <span>{connected ? 'connected' : 'disconnected'}</span>
  </div>

  {#if user}
    <div class="user">Logged in as <b>{user.globalName ?? user.username}</b> ({user.id})</div>
  {/if}

  <details open>
    <summary>Event log</summary>
    <ul class="log">
      {#each log as e}<li><code>{e.t}</code> <b>{e.kind}</b> {e.detail}</li>{/each}
    </ul>
  </details>
</section>
