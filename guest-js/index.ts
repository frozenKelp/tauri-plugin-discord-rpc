import { invoke } from '@tauri-apps/api/core'

export interface Assets {
  largeImage?: string
  largeText?: string
  smallImage?: string
  smallText?: string
}

export interface Button {
  label: string
  url: string
}

export interface Party {
  id?: string
  currentSize?: number
  maxSize?: number
}

export interface Timestamps {
  start?: number
  end?: number
}

export interface Activity {
  state?: string
  details?: string
  assets?: Assets
  buttons?: Button[]
  party?: Party
  timestamps?: Timestamps
  /** 0 Playing, 2 Listening, 3 Watching, 5 Competing. */
  activityType?: number
  /** 0 Name, 1 State, 2 Details — compact headline. */
  statusDisplayType?: number
}

export async function connect(appId: string): Promise<void> {
  await invoke('plugin:discord-rpc|connect', { appId })
}

export async function disconnect(): Promise<void> {
  await invoke('plugin:discord-rpc|disconnect')
}

export async function setActivity(payload: Activity): Promise<void> {
  await invoke('plugin:discord-rpc|set_activity', { payload })
}

export async function clearActivity(): Promise<void> {
  await invoke('plugin:discord-rpc|clear_activity')
}

export async function isRunning(): Promise<boolean> {
  return await invoke('plugin:discord-rpc|is_running')
}
