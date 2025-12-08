import type { AppConfig } from "./config.js";

export const IPC_CHANNELS = {
	CONFIG_DATA: "config-data",
	ABOUT_DATA: "about-data",
	UPDATE_AVAILABLE: "update-available",
	UPDATE_CHECK_FAILED: "update-check-failed",

	SAVE_CONFIG: "save-config",
	CHECK_FOR_UPDATE: "check-for-update",
	OPEN_EXTERNAL: "open-external",
} as const;

export type IpcChannel = (typeof IPC_CHANNELS)[keyof typeof IPC_CHANNELS];

export interface UpdateAvailablePayload {
	available: boolean;
	version?: string;
	downloadUrl?: string;
}

export interface AboutDataPayload {
	version: string;
}

export interface ElectronAPI {
	saveConfig: (config: AppConfig) => void;
	onConfigData: (callback: (config: AppConfig) => void) => () => void;

	onAboutData: (callback: (version: string) => void) => () => void;
	checkForUpdate: () => void;
	onUpdateAvailable: (
		callback: (available: boolean, version?: string, downloadUrl?: string) => void,
	) => () => void;
	onUpdateCheckFailed: (callback: () => void) => () => void;

	openExternal: (url: string) => void;
}

declare global {
	interface Window {
		electronAPI: ElectronAPI;
	}
}
