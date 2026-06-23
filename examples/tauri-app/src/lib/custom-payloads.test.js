import { describe, it, expect } from 'vitest'
import { CUSTOM_TEMPLATES, DISCORD_APP_ID } from './custom-payloads.js'

describe('custom payload templates', () => {
  it('exposes a default app id', () => {
    expect(typeof DISCORD_APP_ID).toBe('string')
    expect(DISCORD_APP_ID.length).toBeGreaterThan(0)
  })

  it('every template has a label and a JSON-serializable activity object', () => {
    expect(CUSTOM_TEMPLATES.length).toBeGreaterThan(0)
    for (const t of CUSTOM_TEMPLATES) {
      expect(typeof t.label).toBe('string')
      const json = JSON.stringify(t.activity)
      expect(() => JSON.parse(json)).not.toThrow()
      expect(typeof JSON.parse(json)).toBe('object')
    }
  })
})
