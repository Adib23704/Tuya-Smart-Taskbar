import { app } from "electron";
import { initConfigManager } from "./config.js";
import { setupIpcHandlers } from "./ipc.js";
import { destroyTray, initTray, startAutoRefresh, updateMenu } from "./tray.js";
import { createTuyaContext } from "./tuya/index.js";

const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
	app.quit();
} else {
	app.on("second-instance", () => {
		console.log("Second instance attempted - already running");
	});

	app.whenReady().then(() => {
		const configManager = initConfigManager();

		configManager.updateStartupSettings();

		const config = configManager.get();
		createTuyaContext(config);

		initTray();

		setupIpcHandlers();

		updateMenu();

		startAutoRefresh();

		console.log("Tuya Smart Taskbar started successfully");
	});

	app.on("window-all-closed", () => {
		// Do nothing - keep running in tray
	});

	app.on("before-quit", () => {
		destroyTray();
	});

	app.on("activate", () => {
		// On macOS, re-create the tray if needed
	});
}
