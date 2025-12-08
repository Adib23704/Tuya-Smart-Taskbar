/**
 * IPC handlers module
 */

import { app, ipcMain, shell } from "electron";
import type { AppConfig } from "../types/config.js";
import { IPC_CHANNELS } from "../types/electron.js";
import { getConfigManager } from "./config.js";
import { updateMenu } from "./tray.js";
import { createTuyaContext } from "./tuya/index.js";

const GITHUB_PACKAGE_URL =
	"https://raw.githubusercontent.com/Adib23704/Tuya-Smart-Taskbar/refs/heads/master/package.json";
const DOWNLOAD_URL = "https://github.com/Adib23704/Tuya-Smart-Taskbar/releases/latest";

interface PackageJson {
	version: string;
}

export function setupIpcHandlers(): void {
	ipcMain.on(IPC_CHANNELS.SAVE_CONFIG, (_event, newConfig: AppConfig) => {
		const configManager = getConfigManager();

		configManager.save(newConfig);

		configManager.updateStartupSettings();

		createTuyaContext(newConfig);

		updateMenu();
	});

	ipcMain.on(IPC_CHANNELS.CHECK_FOR_UPDATE, async (event) => {
		try {
			const response = await fetch(GITHUB_PACKAGE_URL);

			if (!response.ok) {
				throw new Error(`HTTP error: ${response.status}`);
			}

			const data = (await response.json()) as PackageJson;
			const latestVersion = data.version;
			const currentVersion = app.getVersion();

			console.log(`Current version: ${currentVersion}, Latest version: ${latestVersion}`);

			if (latestVersion !== currentVersion) {
				event.sender.send(IPC_CHANNELS.UPDATE_AVAILABLE, true, latestVersion, DOWNLOAD_URL);
			} else {
				event.sender.send(IPC_CHANNELS.UPDATE_AVAILABLE, false);
			}
		} catch (error) {
			console.error("Error checking for update:", error);
			event.sender.send(IPC_CHANNELS.UPDATE_CHECK_FAILED);
		}
	});

	ipcMain.on(IPC_CHANNELS.OPEN_EXTERNAL, (_event, url: string) => {
		try {
			const parsedUrl = new URL(url);
			if (parsedUrl.protocol === "https:" || parsedUrl.protocol === "http:") {
				shell.openExternal(url);
			}
		} catch {
			console.error("Invalid URL:", url);
		}
	});
}

export function removeIpcHandlers(): void {
	ipcMain.removeAllListeners(IPC_CHANNELS.SAVE_CONFIG);
	ipcMain.removeAllListeners(IPC_CHANNELS.CHECK_FOR_UPDATE);
	ipcMain.removeAllListeners(IPC_CHANNELS.OPEN_EXTERNAL);
}
