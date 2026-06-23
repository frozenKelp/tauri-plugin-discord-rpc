'use strict';

var core = require('@tauri-apps/api/core');

async function connect(appId) {
    await core.invoke('plugin:discord-rpc|connect', { appId });
}
async function disconnect() {
    await core.invoke('plugin:discord-rpc|disconnect');
}
async function setActivity(payload) {
    await core.invoke('plugin:discord-rpc|set_activity', { payload });
}
/** Send a raw activity payload straight to Discord (bypasses the typed API). Advanced/experimental. */
async function setActivityRaw(payload) {
    await core.invoke('plugin:discord-rpc|set_activity_raw', { payload });
}
async function clearActivity() {
    await core.invoke('plugin:discord-rpc|clear_activity');
}
async function isConnected() {
    return await core.invoke('plugin:discord-rpc|is_connected');
}
async function getCurrentUser() {
    return await core.invoke('plugin:discord-rpc|get_current_user');
}

exports.clearActivity = clearActivity;
exports.connect = connect;
exports.disconnect = disconnect;
exports.getCurrentUser = getCurrentUser;
exports.isConnected = isConnected;
exports.setActivity = setActivity;
exports.setActivityRaw = setActivityRaw;
