<script>
  import { setActivity, clearActivity } from 'tauri-plugin-discord-rpc-api'
  import { ACTIVITY_TYPES, STATUS_DISPLAY_TYPES, cleanActivity } from './activity-form.js'

  let { connected = false, form = $bindable() } = $props()

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

<div class="demo">
  <div class="fields">
    <label class="field"><span>Type</span>
      <select bind:value={form.activityType}>
        {#each ACTIVITY_TYPES as t}<option value={t.value}>{t.label}</option>{/each}
      </select>
    </label>
    <label class="field"><span>Status display</span>
      <select bind:value={form.statusDisplayType}>
        {#each STATUS_DISPLAY_TYPES as t}<option value={t.value}>{t.label}</option>{/each}
      </select>
    </label>
    <label class="field"><span>Name</span><input bind:value={form.name} /></label>
    <label class="field"><span>Details</span><input bind:value={form.details} /></label>
    <label class="field"><span>Details URL</span><input bind:value={form.detailsUrl} /></label>
    <label class="field"><span>State</span><input bind:value={form.state} /></label>
    <label class="field"><span>State URL</span><input bind:value={form.stateUrl} /></label>
  </div>

  <fieldset>
    <legend>Assets</legend>
    <div class="fields">
      <label class="field"><span>Large image</span><input bind:value={form.assets.largeImage} /></label>
      <label class="field"><span>Large text</span><input bind:value={form.assets.largeText} /></label>
      <label class="field"><span>Large URL</span><input bind:value={form.assets.largeUrl} /></label>
      <label class="field"><span>Small image</span><input bind:value={form.assets.smallImage} /></label>
      <label class="field"><span>Small text</span><input bind:value={form.assets.smallText} /></label>
      <label class="field"><span>Small URL</span><input bind:value={form.assets.smallUrl} /></label>
    </div>
  </fieldset>

  <fieldset>
    <legend>Buttons (max 2)</legend>
    {#each form.buttons as button, i}
      <div class="btn-row">
        <label class="field"><span>Label {i + 1}</span><input bind:value={button.label} /></label>
        <label class="field"><span>URL {i + 1}</span><input bind:value={button.url} /></label>
      </div>
    {/each}
  </fieldset>

  <fieldset>
    <legend>Party</legend>
    <div class="fields">
      <label class="field"><span>Id</span><input bind:value={form.party.id} /></label>
      <label class="field"><span>Current size</span><input type="number" bind:value={form.party.currentSize} /></label>
      <label class="field"><span>Max size</span><input type="number" bind:value={form.party.maxSize} /></label>
    </div>
  </fieldset>

  <div class="actions">
    <button type="button" onclick={apply} disabled={!connected}>Set activity</button>
    <button type="button" onclick={clear} disabled={!connected}>Clear</button>
    {#if status}<span class="msg">{status}</span>{/if}
  </div>

  <details class="payload" open>
    <summary>Payload preview</summary>
    <pre class="preview">{payloadJson}</pre>
  </details>
</div>
