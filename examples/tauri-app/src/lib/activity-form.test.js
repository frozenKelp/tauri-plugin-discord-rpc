import { describe, it, expect } from 'vitest'
import { cleanActivity, createActivityForm, ACTIVITY_TYPES } from './activity-form.js'

describe('activity-form (crate/no-auth surface)', () => {
  it('omits empty fields', () => {
    const form = createActivityForm()
    form.details = ''
    form.state = ''
    const out = cleanActivity(form)
    expect(out.details).toBeUndefined()
    expect(out.state).toBeUndefined()
  })

  it('does NOT emit auth/unsupported fields', () => {
    const out = cleanActivity(createActivityForm())
    expect(out.emoji).toBeUndefined()
    expect(out.secrets).toBeUndefined()
    expect(out.instance).toBeUndefined()
    expect(out.url).toBeUndefined()
  })

  it('only offers crate activity types (0,2,3,5)', () => {
    expect(ACTIVITY_TYPES.map(t => t.value).sort((a,b)=>a-b)).toEqual([0, 2, 3, 5])
  })

  it('keeps crate fields and caps buttons at 2', () => {
    const form = createActivityForm()
    form.state = 'hello'
    form.assets.largeImage = 'key'
    form.buttons = [
      { label: 'a', url: 'https://a' },
      { label: 'b', url: 'https://b' },
      { label: 'c', url: 'https://c' },
    ]
    const out = cleanActivity(form)
    expect(out.state).toBe('hello')
    expect(out.assets.largeImage).toBe('key')
    expect(out.buttons).toHaveLength(2)
  })
})
