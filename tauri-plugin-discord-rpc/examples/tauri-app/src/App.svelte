<script>
  import { connect, disconnect, setActivity, clearActivity, isRunning } from 'tauri-plugin-discord-rpc-api'
  import { listen } from '@tauri-apps/api/event'
  import { onMount, onDestroy } from 'svelte'

  const APP_ID = '1489646398478487773'

  let connState  = $state('idle')
  let running    = $state(false)
  let statusMsg  = $state('')
  let autoWatch  = $state(false)

  let details   = $state('Testing plugin')
  let presState = $state('It works')
  let startTime = $state(true)

  let buttons = $state([
    { label: 'GitHub', url: 'https://github.com' },
    { label: '',       url: '' },
  ])

  const canConnect    = $derived(connState === 'idle' || connState === 'error')
  const canDisconnect = $derived(connState === 'connected' || autoWatch)
  const canSetClear   = $derived(connState === 'connected')

  const validButtons = $derived(
    buttons.filter(b => b.label.trim() && b.url.trim())
  )

  let unlisten
  let stopLoop = false
  const POLL_MS = 3000

  onMount(async () => {
    running = await isRunning().catch(() => false)
    if (running) connState = 'connected'

    unlisten = await listen('discord-rpc://running', ({ payload }) => {
      running = payload
      if (payload) {
        connState = 'connected'
        setStatus('Connected')
      } else if (connState === 'connected') {
        connState = 'idle'
        setStatus('Discord closed — retrying…')
      }
    })
  })

  onDestroy(() => unlisten?.())

  async function startAutoConnect() {
    if (autoWatch) return
    stopLoop  = false
    autoWatch = true
    setStatus('Watching for Discord…')

    while (!stopLoop) {
      if (connState !== 'connected' && connState !== 'connecting') {
        connState = 'connecting'
        setStatus('Connecting…')
        try {
          await connect(APP_ID)
          if (stopLoop) await disconnect().catch(() => {})
        } catch {
          connState = 'idle'
        }
      }
      if (!stopLoop) await delay(POLL_MS)
    }

    autoWatch = false
  }

  function stopAutoConnect() {
    stopLoop = true
    setStatus('Watcher stopped')
  }

  async function handleConnect() {
    connState = 'connecting'
    setStatus('Connecting…')
    try {
      await connect(APP_ID)
      running   = true
      connState = 'connected'
      setStatus('Connected')
    } catch (e) {
      connState = 'error'
      setStatus(`${e}`)
    }
  }

  async function handleDisconnect() {
    connState = 'disconnecting'
    stopLoop  = true
    try {
      await disconnect()
    } catch {}
    running   = false
    connState = 'idle'
    autoWatch = false
    setStatus('Disconnected')
  }

  async function handleSetActivity() {
    try {
      await setActivity({
        details:    details   || undefined,
        state:      presState || undefined,
        timestamps: startTime ? { start: Date.now() } : undefined,
        buttons:    validButtons.length ? validButtons : undefined,
      })
      setStatus('Activity set ✓')
    } catch (e) {
      setStatus(`Error: ${e}`)
    }
  }

  async function handleClear() {
    try {
      await clearActivity()
      setStatus('Activity cleared')
    } catch (e) {
      setStatus(`Error: ${e}`)
    }
  }

  function setStatus(msg) { statusMsg = msg }
  const delay = ms => new Promise(r => setTimeout(r, ms))
</script>

<main>
  <h1>Discord RPC</h1>

  <div class="status-bar">
    <span class="dot" class:green={connState === 'connected'} class:yellow={connState === 'connecting' || connState === 'disconnecting'} class:red={connState === 'error'}></span>
    <span>{connState}</span>
    {#if statusMsg}
      <span class="sep">·</span>
      <span class="msg">{statusMsg}</span>
    {/if}
  </div>

  <section>
    <h2>Connection</h2>
    <div class="row">
      <button onclick={handleConnect} disabled={!canConnect || autoWatch}>Connect</button>
      <button onclick={handleDisconnect} disabled={connState === 'idle' && !autoWatch}>Disconnect</button>
      {#if !autoWatch}
        <button class="accent" onclick={startAutoConnect}>▶ Start watcher</button>
      {:else}
        <button class="danger" onclick={stopAutoConnect}>■ Stop watcher</button>
      {/if}
    </div>
    <p class="hint"><strong>Watcher</strong> auto-connects when Discord opens and re-connects if it closes.</p>
  </section>

  <section>
    <h2>Rich Presence</h2>

    <label>
      Details
      <input type="text" bind:value={details} placeholder="What you're doing" />
    </label>

    <label>
      State
      <input type="text" bind:value={presState} placeholder="Party / status line" />
    </label>

    <label class="checkbox-label">
      <input type="checkbox" bind:checked={startTime} />
      Include elapsed timer (start = now)
    </label>

    <h3>Buttons <span class="hint-inline">(max 2, both fields required)</span></h3>
    {#each buttons as btn, i}
      <div class="btn-row">
        <span class="btn-num">{i + 1}</span>
        <input type="text" bind:value={btn.label} placeholder="Label" maxlength="32" />
        <input type="url"  bind:value={btn.url}   placeholder="https://…" />
      </div>
    {/each}

    <div class="row" style="margin-top:1rem">
      <button onclick={handleSetActivity} disabled={!canSetClear}>Set activity</button>
      <button onclick={handleClear}       disabled={!canSetClear}>Clear activity</button>
    </div>
  </section>
</main>

<style>
  main {
    max-width: 540px;
    margin: 0 auto;
    padding: 2rem 1.5rem;
  }

  h1 { text-align: center; margin-bottom: 1.5rem; }
  h2 { margin: 0 0 .75rem; font-size: 1rem; text-transform: uppercase; letter-spacing: .05em; opacity: .6; }
  h3 { margin: 1rem 0 .4rem; font-size: .9rem; }

  section {
    background: rgba(255,255,255,.06);
    border: 1px solid rgba(255,255,255,.1);
    border-radius: 10px;
    padding: 1.2rem 1.4rem;
    margin-bottom: 1.2rem;
  }

  .status-bar {
    display: flex;
    align-items: center;
    gap: .5rem;
    margin-bottom: 1.2rem;
    font-size: .9rem;
    opacity: .85;
  }
  .dot {
    width: 9px; height: 9px;
    border-radius: 50%;
    background: #555;
    flex-shrink: 0;
  }
  .dot.green  { background: #3ba55d; }
  .dot.yellow { background: #faa61a; }
  .dot.red    { background: #ed4245; }
  .sep { opacity: .4; }
  .msg { opacity: .7; }

  .row { display: flex; gap: .6rem; flex-wrap: wrap; }

  button { flex-shrink: 0; }
  button.accent  { background: #5865f2; color: #fff; border-color: transparent; }
  button.danger  { background: #ed4245; color: #fff; border-color: transparent; }
  button:disabled { opacity: .35; cursor: not-allowed; }

  label {
    display: flex;
    flex-direction: column;
    gap: .25rem;
    font-size: .85rem;
    font-weight: 600;
    margin-bottom: .75rem;
  }

  .checkbox-label { flex-direction: row; align-items: center; gap: .5rem; }
  .checkbox-label input { width: auto; }

  input[type="text"],
  input[type="url"] { width: 100%; box-sizing: border-box; }

  .btn-row { display: flex; align-items: center; gap: .5rem; margin-bottom: .5rem; }
  .btn-num { font-size: .75rem; opacity: .4; width: 14px; text-align: center; flex-shrink: 0; }
  .btn-row input[type="text"] { flex: 1; min-width: 0; }
  .btn-row input[type="url"]  { flex: 2; min-width: 0; }

  .hint { font-size: .8rem; opacity: .5; margin: .6rem 0 0; }
  .hint-inline { font-size: .75rem; opacity: .55; font-weight: 400; }
</style>