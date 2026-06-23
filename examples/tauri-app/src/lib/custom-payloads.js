export const DISCORD_APP_ID = '1487894643613106298'

// Raw activity payloads for the Custom Payload tab. These bypass the typed API via setActivityRaw
// and intentionally include fields the plugin does NOT model (emoji, secrets, instance, type 4,
// top-level url). Some require OAuth, which this plugin does not do — they may be ignored/rejected.
export const CUSTOM_TEMPLATES = [
  {
    label: 'Watching (baseline, supported)',
    activity: { type: 3, details: 'Manual probe', state: 'Baseline raw send', timestamps: { start: Date.now() } },
  },
  {
    label: 'Custom status w/ emoji (type 4 — experimental)',
    activity: { type: 4, state: 'Custom raw probe', emoji: { name: '✨' } },
  },
  {
    label: 'Secrets + instance (needs OAuth — likely rejected)',
    activity: {
      type: 0,
      details: 'Secrets probe',
      state: 'Ask-to-join / spectate',
      party: { id: 'probe-party', size: [1, 4] },
      secrets: { join: 'probe-join-secret' },
      instance: true,
    },
  },
  {
    label: 'Rich payload (urls + buttons)',
    activity: {
      type: 0,
      details: 'Raw rich payload',
      state: 'Urls and buttons',
      assets: { large_image: 'embedded_cover', large_text: 'Large', large_url: 'https://example.com' },
      buttons: [{ label: 'Docs', url: 'https://discord.com/developers/docs' }],
    },
  },
]
