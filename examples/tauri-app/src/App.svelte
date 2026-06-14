<script>
  import ConnectionBar from './lib/ConnectionBar.svelte'
  import EventLog from './lib/EventLog.svelte'
  import PluginDemo from './lib/PluginDemo.svelte'
  import CustomPayload from './lib/CustomPayload.svelte'
  import { createActivityForm } from './lib/activity-form.js'
  import { CUSTOM_TEMPLATES } from './lib/custom-payloads.js'

  let connected = $state(false)
  let tab = $state('demo')

  // Editor state survives tab switches here
  // save slots can snapshot/restore it.
  let demoForm = $state(createActivityForm())
  // Wrapped in an object so the editor text can be mutated in place (the child shares this proxy).
  let raw = $state({ text: JSON.stringify(CUSTOM_TEMPLATES[0].activity, null, 2) })

  // Six in-memory save slots. Each null or { tab, data } where data is a form snapshot or raw text.
  let slots = $state(Array(6).fill(null))
  let note = $state('')

  // Portable deep copy of plain JSON-safe data (avoids structuredClone, which some webviews lack).
  /** @param {any} v */
  const clone = (v) => JSON.parse(JSON.stringify(v))

  /** @param {number} i */
  function saveSlot(i) {
    slots[i] = tab === 'demo'
      ? { tab, data: clone(demoForm) }
      : { tab, data: raw.text }
    note = `Saved current ${tab === 'demo' ? 'form' : 'payload'} → slot ${i + 1}`
  }
  /** @param {number} i */
  function loadSlot(i) {
    const slot = slots[i]
    if (!slot) { note = `Slot ${i + 1} is empty — double-click to save`; return }
    tab = slot.tab
    // Mutate the shared state in place (don't reassign the bound prop) so the child's field
    // bindings pick it up. clone() keeps the stored slot from aliasing the live editor.
    if (slot.tab === 'demo') Object.assign(demoForm, clone(slot.data))
    else raw.text = slot.data
    note = `Loaded slot ${i + 1} (${slot.tab === 'demo' ? 'form' : 'payload'})`
  }

  // Single click = load, double click = save. Defer the click briefly so a double-click cancels it.
  /** @type {ReturnType<typeof setTimeout> | undefined} */
  let clickTimer
  /** @param {number} i */
  function onSlotClick(i) {
    if (clickTimer) return
    clickTimer = setTimeout(() => { clickTimer = undefined; loadSlot(i) }, 250)
  }
  /** @param {number} i */
  function onSlotDblClick(i) {
    clearTimeout(clickTimer); clickTimer = undefined
    saveSlot(i)
  }
</script>

<main>
  <h1>tauri-plugin-discord-rpc — TEST</h1>
  <div class="layout">
    <section class="content">
      <ConnectionBar onConnectedChange={(v) => (connected = v)} />

      <nav class="tabs">
        <button type="button" class:active={tab === 'demo'} onclick={() => (tab = 'demo')}>Plugin Demo</button>
        <button type="button" class:active={tab === 'custom'} onclick={() => (tab = 'custom')}>Custom Payload</button>
        <span class="slot-sep" aria-hidden="true"></span>
        {#each slots as slot, i}
          <button
            type="button"
            class="slot"
            class:filled={slot}
            title={slot
              ? `Slot ${i + 1} (${slot.tab}) — click to load, double-click to overwrite`
              : `Slot ${i + 1} empty — double-click to save current state`}
            onclick={() => onSlotClick(i)}
            ondblclick={() => onSlotDblClick(i)}
          >{i + 1}</button>
        {/each}
      </nav>

      {#if note}<p class="slot-note">{note}</p>{/if}

      {#if tab === 'demo'}
        <PluginDemo {connected} bind:form={demoForm} />
      {:else}
        <CustomPayload {connected} bind:raw={raw} />
      {/if}
    </section>

    <EventLog />
  </div>
</main>
