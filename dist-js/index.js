import { invoke } from '@tauri-apps/api/core';

async function connect(appId) {
    await invoke('plugin:discord-rpc|connect', { appId });
}
async function disconnect() {
    await invoke('plugin:discord-rpc|disconnect');
}
async function setActivity(payload) {
    await invoke('plugin:discord-rpc|set_activity', { payload });
}
async function clearActivity() {
    await invoke('plugin:discord-rpc|clear_activity');
}
async function isRunning() {
    return await invoke('plugin:discord-rpc|is_running');
}

export { clearActivity, connect, disconnect, isRunning, setActivity };
