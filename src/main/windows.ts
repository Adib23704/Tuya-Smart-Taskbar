import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { app, BrowserWindow } from "electron";
import { getConfigManager } from "./config.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

let configWindow: BrowserWindow | null = null;
let aboutWindow: BrowserWindow | null = null;

function getAppRoot(): string {
	return app.isPackaged ? join(process.resourcesPath, "app") : join(__dirname, "..", "..");
}

function getIconPath(): string {
	return join(getAppRoot(), "assets", "icon.ico");
}

function getPreloadPath(): string {
	return join(__dirname, "..", "preload", "index.js");
}

const WINDOW_CONFIG = {
	config: {
		width: 400,
		height: 580,
		title: "Tuya Smart Taskbar Config",
	},
	about: {
		width: 400,
		height: 500,
		title: "About Tuya Smart Taskbar",
	},
} as const;

function createWindow(type: "config" | "about"): BrowserWindow {
	const windowConfig = WINDOW_CONFIG[type];

	const window = new BrowserWindow({
		width: windowConfig.width,
		height: windowConfig.height,
		resizable: false,
		webPreferences: {
			nodeIntegration: false,
			contextIsolation: true,
			preload: getPreloadPath(),
			sandbox: true,
		},
		title: windowConfig.title,
		icon: getIconPath(),
		autoHideMenuBar: true,
		center: true,
		fullscreenable: false,
		movable: true,
		show: false,
	});

	window.once("ready-to-show", () => {
		window.show();
	});

	return window;
}

export function openConfigWindow(): void {
	if (configWindow) {
		configWindow.focus();
		return;
	}

	configWindow = createWindow("config");

	const htmlPath = join(getAppRoot(), "pages", "config.html");
	configWindow.loadFile(htmlPath);

	configWindow.on("closed", () => {
		configWindow = null;
	});

	configWindow.webContents.on("did-finish-load", () => {
		const config = getConfigManager().get();
		configWindow?.webContents.send("config-data", config);
	});
}

export function openAboutWindow(): void {
	if (aboutWindow) {
		aboutWindow.focus();
		return;
	}

	aboutWindow = createWindow("about");

	const htmlPath = join(getAppRoot(), "pages", "about.html");
	aboutWindow.loadFile(htmlPath);

	aboutWindow.on("closed", () => {
		aboutWindow = null;
	});

	aboutWindow.webContents.on("did-finish-load", () => {
		const version = app.getVersion();
		aboutWindow?.webContents.send("about-data", version);
	});
}

export function getConfigWindow(): BrowserWindow | null {
	return configWindow;
}

export function getAboutWindow(): BrowserWindow | null {
	return aboutWindow;
}

export function closeAllWindows(): void {
	if (configWindow) {
		configWindow.close();
		configWindow = null;
	}
	if (aboutWindow) {
		aboutWindow.close();
		aboutWindow = null;
	}
}
