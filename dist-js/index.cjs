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
async function isRunning() {
    return await core.invoke('plugin:discord-rpc|is_running');
}

exports.clearActivity = clearActivity;
exports.connect = connect;
exports.disconnect = disconnect;
exports.isRunning = isRunning;
exports.setActivity = setActivity;
