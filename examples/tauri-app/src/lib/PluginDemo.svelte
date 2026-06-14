<script>
  import { setActivity, clearActivity } from 'tauri-plugin-discord-rpc-api'
  import { ACTIVITY_TYPES, STATUS_DISPLAY_TYPES, createActivityForm, cleanActivity } from './activity-form.js'

  let { connected = false } = $props()

  let form = $state(createActivityForm())
  let status = $state('')
  const payload = $derived(cleanActivity(form))
  const payloadJson = $derived(JSON.stringify(payload, null, 2))

  async function apply() {
    try { await setActivity(payload); status = 'Activity set' }
    catch (e) { status = `Error: ${e}` }
  }
  async function clear() {
    try { await clearActivity(); status = 'Cleared' }
    catch (e) { status = `Error: ${e}` }
  }
</script>

<div class="grid">
  <form onsubmit={(e) => e.preventDefault()}>
    <label>Type
      <select bind:value={form.activityType}>
        {#each ACTIVITY_TYPES as t}<option value={t.value}>{t.label}</option>{/each}
      </select>
    </label>
    <label>Status display
      <select bind:value={form.statusDisplayType}>
        {#each STATUS_DISPLAY_TYPES as t}<option value={t.value}>{t.label}</option>{/each}
      </select>
    </label>
    <label>Name <input bind:value={form.name} /></label>
    <label>Details <input bind:value={form.details} /></label>
    <label>Details URL <input bind:value={form.detailsUrl} /></label>
    <label>State <input bind:value={form.state} /></label>
    <label>State URL <input bind:value={form.stateUrl} /></label>
    <fieldset><legend>Assets</legend>
      <label>Large image <input bind:value={form.assets.largeImage} /></label>
      <label>Large text <input bind:value={form.assets.largeText} /></label>
      <label>Large URL <input bind:value={form.assets.largeUrl} /></label>
      <label>Small image <input bind:value={form.assets.smallImage} /></label>
      <label>Small text <input bind:value={form.assets.smallText} /></label>
      <label>Small URL <input bind:value={form.assets.smallUrl} /></label>
    </fieldset>
    <fieldset><legend>Buttons (max 2)</legend>
      {#each form.buttons as btn, i}
        <label>Label <input bind:value={form.buttons[i].label} /></label>
        <label>URL <input bind:value={form.buttons[i].url} /></label>
      {/each}
    </fieldset>
    <fieldset><legend>Party</legend>
      <label>Id <input bind:value={form.party.id} /></label>
      <label>Current <input type="number" bind:value={form.party.currentSize} /></label>
      <label>Max <input type="number" bind:value={form.party.maxSize} /></label>
    </fieldset>
    <div class="actions">
      <button onclick={apply} disabled={!connected}>Set activity</button>
      <button onclick={clear} disabled={!connected}>Clear</button>
      <span>{status}</span>
    </div>
  </form>
  <pre class="preview">{payloadJson}</pre>
</div>
