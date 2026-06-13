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
