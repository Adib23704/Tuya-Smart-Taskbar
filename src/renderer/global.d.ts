
interface AppConfig {
	baseUrl: string;
	accessKey: string;
	secretKey: string;
	userId: string;
	runOnStartup: boolean;
}

interface ElectronAPI {
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

interface Window {
	electronAPI: ElectronAPI;
}
