<script>
  import ConnectionBar from './lib/ConnectionBar.svelte'
  import PluginDemo from './lib/PluginDemo.svelte'
  import CustomPayload from './lib/CustomPayload.svelte'

  let connected = $state(false)
  let tab = $state('demo')
</script>

<main>
  <h1>tauri-plugin-discord-rpc — example</h1>
  <ConnectionBar onConnectedChange={(v) => (connected = v)} />

  <nav class="tabs">
    <button class:active={tab === 'demo'} onclick={() => (tab = 'demo')}>Plugin Demo</button>
    <button class:active={tab === 'custom'} onclick={() => (tab = 'custom')}>Custom Payload</button>
  </nav>

  {#if tab === 'demo'}
    <PluginDemo {connected} />
  {:else}
    <CustomPayload {connected} />
  {/if}
</main>
