<script>
  import { setActivityRaw } from 'tauri-plugin-discord-rpc-api'

  let { connected = false, raw = $bindable() } = $props()

  let status = $state('')

  async function send() {
    let parsed
    try { parsed = JSON.parse(raw.text) }
    catch (e) { status = `Invalid JSON: ${e}`; return }
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
      status = 'Payload must be a JSON object'; return
    }
    try { await setActivityRaw(parsed); status = 'Raw payload sent — watch the event log for rejections' }
    catch (e) { status = `Error: ${e}` }
  }
</script>

<div class="custom">
  <label class="raw-label" for="raw-json">Raw activity - JSON</label>
  <textarea id="raw-json" bind:value={raw.text} rows="16" spellcheck="false"></textarea>
  <div class="actions">
    <button type="button" onclick={send} disabled={!connected}>Send raw</button>
    {#if status}<span class="msg">{status}</span>{/if}
  </div>

  <div class="help">
    <p>
      send JSON straight to Discord's <code>SET_ACTIVITY</code> via
      <code>setActivityRaw</code>, bypassing crate and API — handy for checking what Discord accepts.
      Keys use Discord's <strong>snake_case</strong> wire format (<code>large_image</code>, not
      <code>largeImage</code>).
    </p>

    <p class="help-h">Commonly supported fields</p>
    <ul>
      <li><code>type</code> — 0 Playing · 2 Listening · 3 Watching · 5 Competing</li>
      <li><code>details</code>, <code>state</code>, <code>details_url</code>, <code>state_url</code></li>
      <li><code>timestamps</code>: <code>{`{ start, end }`}</code> (epoch milliseconds)</li>
      <li><code>assets</code>: <code>{`{ large_image, large_text, large_url, small_image, small_text, small_url }`}</code></li>
      <li><code>buttons</code>: <code>{`[{ label, url }]`}</code> (max 2)</li>
      <li><code>party</code>: <code>{`{ id, size: [current, max] }`}</code></li>
    </ul>

    <p class="warn">
      <strong>Experimental / needs OAuth</strong> (this plugin has no OAuth, so Discord typically
      ignores these): <code>secrets</code>, <code>instance</code>.
      When Discord rejects a payload, the reason shows up in the <strong>event log</strong> on the right.
    </p>
  </div>
</div>
