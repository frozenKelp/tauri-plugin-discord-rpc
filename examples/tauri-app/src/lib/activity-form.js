export const ACTIVITY_TYPES = [
  { value: 0, label: 'Playing' },
  { value: 2, label: 'Listening' },
  { value: 3, label: 'Watching' },
  { value: 5, label: 'Competing' },
]

export const STATUS_DISPLAY_TYPES = [
  { value: '', label: 'Omit' },
  { value: 0, label: 'Name' },
  { value: 1, label: 'State' },
  { value: 2, label: 'Details' },
]

export function createActivityForm() {
  return {
    activityType: 3,
    statusDisplayType: '',
    name: '',
    details: 'Testing plugin',
    detailsUrl: '',
    state: 'It works',
    stateUrl: '',
    timestamps: { start: Date.now(), end: '' },
    assets: {
      largeImage: '', largeText: '', largeUrl: '',
      smallImage: '', smallText: '', smallUrl: '',
    },
    buttons: [
      { label: 'GitHub', url: 'https://github.com' },
      { label: '', url: '' },
    ],
    party: { id: '', currentSize: '', maxSize: '' },
  }
}

export function cleanActivity(form) {
  return cleanObject({
    activityType: numberOrUndefined(form.activityType),
    statusDisplayType: numberOrUndefined(form.statusDisplayType),
    name: textOrUndefined(form.name),
    details: textOrUndefined(form.details),
    detailsUrl: textOrUndefined(form.detailsUrl),
    state: textOrUndefined(form.state),
    stateUrl: textOrUndefined(form.stateUrl),
    timestamps: cleanObject({
      start: numberOrUndefined(form.timestamps.start),
      end: numberOrUndefined(form.timestamps.end),
    }),
    assets: cleanObject({
      largeImage: textOrUndefined(form.assets.largeImage),
      largeText: textOrUndefined(form.assets.largeText),
      largeUrl: textOrUndefined(form.assets.largeUrl),
      smallImage: textOrUndefined(form.assets.smallImage),
      smallText: textOrUndefined(form.assets.smallText),
      smallUrl: textOrUndefined(form.assets.smallUrl),
    }),
    buttons: completeButtons(form.buttons),
    party: cleanObject({
      id: textOrUndefined(form.party.id),
      currentSize: numberOrUndefined(form.party.currentSize),
      maxSize: numberOrUndefined(form.party.maxSize),
    }),
  })
}

function cleanObject(value) {
  const entries = Object.entries(value).filter(([, item]) => {
    if (item === undefined) return false
    if (Array.isArray(item) && item.length === 0) return false
    if (isPlainObject(item) && Object.keys(item).length === 0) return false
    return true
  })
  return Object.fromEntries(entries)
}

function completeButtons(buttons) {
  const complete = buttons
    .map(button => ({ label: textOrUndefined(button.label), url: textOrUndefined(button.url) }))
    .filter(button => button.label && button.url)
    .slice(0, 2)
  return complete.length ? complete : undefined
}

function textOrUndefined(value) {
  if (value === undefined || value === null) return undefined
  const text = String(value).trim()
  return text ? text : undefined
}

function numberOrUndefined(value) {
  if (value === '' || value === undefined || value === null) return undefined
  const number = Number(value)
  return Number.isFinite(number) ? number : undefined
}

function isPlainObject(value) {
  return value && typeof value === 'object' && !Array.isArray(value)
}
