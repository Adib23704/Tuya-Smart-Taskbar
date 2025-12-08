import { contextBridge, ipcRenderer } from "electron";

const IPC_CHANNELS = {
	CONFIG_DATA: "config-data",
	ABOUT_DATA: "about-data",
	UPDATE_AVAILABLE: "update-available",
	UPDATE_CHECK_FAILED: "update-check-failed",
	SAVE_CONFIG: "save-config",
	CHECK_FOR_UPDATE: "check-for-update",
	OPEN_EXTERNAL: "open-external",
} as const;

interface AppConfig {
	baseUrl: string;
	accessKey: string;
	secretKey: string;
	userId: string;
	runOnStartup: boolean;
}

type CleanupFunction = () => void;

contextBridge.exposeInMainWorld("electronAPI", {
	saveConfig: (config: AppConfig): void => {
		ipcRenderer.send(IPC_CHANNELS.SAVE_CONFIG, config);
	},

	onConfigData: (callback: (config: AppConfig) => void): CleanupFunction => {
		const handler = (_event: Electron.IpcRendererEvent, config: AppConfig) => {
			callback(config);
		};
		ipcRenderer.on(IPC_CHANNELS.CONFIG_DATA, handler);

		return () => {
			ipcRenderer.removeListener(IPC_CHANNELS.CONFIG_DATA, handler);
		};
	},

	onAboutData: (callback: (version: string) => void): CleanupFunction => {
		const handler = (_event: Electron.IpcRendererEvent, version: string) => {
			callback(version);
		};
		ipcRenderer.on(IPC_CHANNELS.ABOUT_DATA, handler);

		return () => {
			ipcRenderer.removeListener(IPC_CHANNELS.ABOUT_DATA, handler);
		};
	},

	checkForUpdate: (): void => {
		ipcRenderer.send(IPC_CHANNELS.CHECK_FOR_UPDATE);
	},

	onUpdateAvailable: (
		callback: (available: boolean, version?: string, downloadUrl?: string) => void,
	): CleanupFunction => {
		const handler = (
			_event: Electron.IpcRendererEvent,
			available: boolean,
			version?: string,
			downloadUrl?: string,
		) => {
			callback(available, version, downloadUrl);
		};
		ipcRenderer.on(IPC_CHANNELS.UPDATE_AVAILABLE, handler);

		return () => {
			ipcRenderer.removeListener(IPC_CHANNELS.UPDATE_AVAILABLE, handler);
		};
	},

	onUpdateCheckFailed: (callback: () => void): CleanupFunction => {
		const handler = () => {
			callback();
		};
		ipcRenderer.on(IPC_CHANNELS.UPDATE_CHECK_FAILED, handler);

		return () => {
			ipcRenderer.removeListener(IPC_CHANNELS.UPDATE_CHECK_FAILED, handler);
		};
	},

	openExternal: (url: string): void => {
		ipcRenderer.send(IPC_CHANNELS.OPEN_EXTERNAL, url);
	},
});
