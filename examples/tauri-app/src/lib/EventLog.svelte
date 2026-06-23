<script>
  import { listen } from '@tauri-apps/api/event'
  import { onMount, onDestroy } from 'svelte'

  let log = $state([])
  const unlisteners = []

  function add(kind, detail) {
    log = [{ t: new Date().toLocaleTimeString(), kind, detail }, ...log].slice(0, 100)
  }

  onMount(async () => {
    unlisteners.push(await listen('discord-rpc://connected', ({ payload }) =>
      add('connected', payload ? 'IPC linked to Discord' : 'connection dropped / Disconnected')))
    unlisteners.push(await listen('discord-rpc://ready', ({ payload }) => {
      const name = payload?.globalName ?? payload?.username ?? 'unknown'
      const handle = payload?.discriminator && payload.discriminator !== '0' ? `${payload.username}#${payload.discriminator}` : payload?.username
      add('ready', `handshake ok`)
    }))
    unlisteners.push(await listen('discord-rpc://error', ({ payload }) => add('error', `Discord rejected update: ${payload}`)))
  })
  onDestroy(() => unlisteners.forEach(u => u?.()))
</script>

<aside class="eventlog">
  <h2>Event log</h2>
  <ul>
    {#each log as e}
      <li class="evt evt-{e.kind}">
        <time>{e.t}</time>
        <span class="kind">{e.kind}</span>
        <span class="detail">{e.detail}</span>
      </li>
    {:else}
      <li class="empty">No events yet — connect to start.</li>
    {/each}
  </ul>
</aside>
