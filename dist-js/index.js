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
async function isConnected() {
    return await invoke('plugin:discord-rpc|is_connected');
}
async function getCurrentUser() {
    return await invoke('plugin:discord-rpc|get_current_user');
}

export { clearActivity, connect, disconnect, getCurrentUser, isConnected, setActivity };
