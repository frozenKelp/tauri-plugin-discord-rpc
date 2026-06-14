<script>
  import { setActivityRaw } from 'tauri-plugin-discord-rpc-api'
  import { CUSTOM_TEMPLATES } from './custom-payloads.js'

  let { connected = false } = $props()

  let text = $state(JSON.stringify(CUSTOM_TEMPLATES[0].activity, null, 2))
  let status = $state('')

  function load(tpl) { text = JSON.stringify(tpl.activity, null, 2); status = `Loaded: ${tpl.label}` }

  async function send() {
    let parsed
    try { parsed = JSON.parse(text) }
    catch (e) { status = `Invalid JSON: ${e}`; return }
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
      status = 'Payload must be a JSON object'; return
    }
    try { await setActivityRaw(parsed); status = 'Raw payload sent (watch the event log for rejections)' }
    catch (e) { status = `Error: ${e}` }
  }
</script>

<div class="custom">
  <p class="warn">
    Experimental — bypasses the typed API via <code>setActivityRaw</code>. Fields like
    <code>secrets</code> require OAuth, which this plugin does not implement, so Discord may ignore
    or reject them. Watch the event log above.
  </p>
  <div class="templates">
    {#each CUSTOM_TEMPLATES as tpl}<button onclick={() => load(tpl)}>{tpl.label}</button>{/each}
  </div>
  <textarea bind:value={text} rows="16" spellcheck="false"></textarea>
  <div class="actions">
    <button onclick={send} disabled={!connected}>Send raw</button>
    <span>{status}</span>
  </div>
</div>
