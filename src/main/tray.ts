import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { MenuItemConstructorOptions } from "electron";
import { app, Menu, Tray } from "electron";
import type { TuyaDevice, TuyaDeviceStatus } from "../types/tuya.js";
import { AC_MODES, type AcMode } from "../types/tuya.js";
import { getConfigManager } from "./config.js";
import { fetchDeviceStatus, fetchDevices, sendDeviceCommand } from "./tuya/index.js";
import { openAboutWindow, openConfigWindow } from "./windows.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

let tray: Tray | null = null;
let autoRefreshInterval: ReturnType<typeof setInterval> | null = null;

const AUTO_REFRESH_INTERVAL = 5000; // 5 seconds
const FAN_SPEED_LEVELS = 5;
const AC_FAN_SPEED_LEVELS = 4;
const TEMP_MIN = 16;
const TEMP_MAX = 30;

function getAppRoot(): string {
	return app.isPackaged ? join(process.resourcesPath, "app") : join(__dirname, "..", "..");
}

function getDefaultIconPath(): string {
	return join(getAppRoot(), "assets", "icon.ico");
}

function getLoadingIconPath(): string {
	return join(getAppRoot(), "assets", "loading.ico");
}

export function initTray(): Tray {
	tray = new Tray(getDefaultIconPath());
	tray.setToolTip("Tuya Smart Taskbar");
	return tray;
}

export function getTray(): Tray | null {
	return tray;
}

export function setTrayIconLoading(loading: boolean): void {
	if (!tray) {
		return;
	}
	tray.setImage(loading ? getLoadingIconPath() : getDefaultIconPath());
}

function formatLabel(code: string): string {
	return code
		.split("_")
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(" ");
}

function createDeviceMenu(
	device: TuyaDevice,
	status: TuyaDeviceStatus[],
): MenuItemConstructorOptions {
	const menuItems: MenuItemConstructorOptions[] = status
		.map((s): MenuItemConstructorOptions | null => {
			if (typeof s.value === "boolean") {
				return {
					label: formatLabel(s.code),
					type: "checkbox",
					checked: s.value,
					enabled: true,
					click: async () => {
						await sendDeviceCommand(device.id, s.code, !s.value);
						await updateMenu();
					},
				};
			}

			if (s.code === "fan_speed_percent") {
				const currentSpeed =
					typeof s.value === "string" ? Number.parseInt(s.value, 10) : Number(s.value);

				return {
					label: "Fan Speed",
					submenu: Array.from(
						{ length: FAN_SPEED_LEVELS },
						(_, i): MenuItemConstructorOptions => ({
							label: `${i + 1}`,
							type: "checkbox",
							checked: currentSpeed === i + 1,
							click: async () => {
								await sendDeviceCommand(device.id, s.code, String(i + 1));
								await updateMenu();
							},
						}),
					),
				};
			}

			if (s.code === "temp_set") {
				const currentTemp = Number(s.value);

				return {
					label: "Temperature",
					submenu: Array.from(
						{ length: TEMP_MAX - TEMP_MIN + 1 },
						(_, i): MenuItemConstructorOptions => ({
							label: `${TEMP_MIN + i}°C`,
							type: "checkbox",
							checked: currentTemp === TEMP_MIN + i,
							click: async () => {
								await sendDeviceCommand(device.id, s.code, TEMP_MIN + i);
								await updateMenu();
							},
						}),
					),
				};
			}

			if (s.code === "windspeed") {
				const currentSpeed =
					typeof s.value === "string" ? Number.parseInt(s.value, 10) : Number(s.value);

				return {
					label: "AC Fan Speed",
					submenu: Array.from(
						{ length: AC_FAN_SPEED_LEVELS },
						(_, i): MenuItemConstructorOptions => ({
							label: `${i + 1}`,
							type: "checkbox",
							checked: currentSpeed === i + 1,
							click: async () => {
								await sendDeviceCommand(device.id, s.code, String(i + 1));
								await updateMenu();
							},
						}),
					),
				};
			}

			if (s.code === "mode") {
				return {
					label: "AC Mode",
					submenu: AC_MODES.map(
						(mode: AcMode): MenuItemConstructorOptions => ({
							label: mode.charAt(0).toUpperCase() + mode.slice(1),
							type: "checkbox",
							checked: s.value === mode,
							click: async () => {
								await sendDeviceCommand(device.id, s.code, mode);
								await updateMenu();
							},
						}),
					),
				};
			}

			return null;
		})
		.filter((item): item is MenuItemConstructorOptions => item !== null);

	return {
		label: device.name,
		submenu: menuItems.length > 0 ? menuItems : [{ label: "No controls", enabled: false }],
	};
}

export async function updateMenu(isAutoRefresh = false): Promise<void> {
	if (!tray) {
		return;
	}

	const configManager = getConfigManager();
	let menuItems: MenuItemConstructorOptions[];

	if (!configManager.isConfigured()) {
		menuItems = [
			{
				label: "Open Configuration",
				click: openConfigWindow,
			},
			{ type: "separator" },
			{ label: "Quit", role: "quit" },
		];
	} else {
		if (!isAutoRefresh) {
			setTrayIconLoading(true);
		}

		try {
			const config = configManager.get();
			const devices = await fetchDevices(config.userId);

			const deviceMenus = await Promise.all(
				devices.map(async (device: TuyaDevice): Promise<MenuItemConstructorOptions | null> => {
					if (!device.online) {
						return null;
					}

					const status = await fetchDeviceStatus(device.id);
					return createDeviceMenu(device, status);
				}),
			);

			const onlineDeviceMenus = deviceMenus.filter(
				(item): item is MenuItemConstructorOptions => item !== null,
			);

			menuItems = [
				...onlineDeviceMenus,
				...(onlineDeviceMenus.length > 0 ? [{ type: "separator" as const }] : []),
				{
					label: "Open Configuration",
					click: openConfigWindow,
				},
				{
					label: "About",
					click: openAboutWindow,
				},
				{ type: "separator" },
				{ label: "Quit", role: "quit" },
			];
		} catch (error) {
			console.error("Error updating menu:", error);
			menuItems = [
				{ label: "Error loading devices", enabled: false },
				{ type: "separator" },
				{
					label: "Open Configuration",
					click: openConfigWindow,
				},
				{ label: "Quit", role: "quit" },
			];
		} finally {
			if (!isAutoRefresh) {
				setTrayIconLoading(false);
			}
		}
	}

	tray.setContextMenu(Menu.buildFromTemplate(menuItems));
}

export function startAutoRefresh(): void {
	if (autoRefreshInterval) {
		clearInterval(autoRefreshInterval);
	}

	autoRefreshInterval = setInterval(async () => {
		await updateMenu(true);
	}, AUTO_REFRESH_INTERVAL);
}

export function stopAutoRefresh(): void {
	if (autoRefreshInterval) {
		clearInterval(autoRefreshInterval);
		autoRefreshInterval = null;
	}
}

export function destroyTray(): void {
	stopAutoRefresh();
	if (tray) {
		tray.destroy();
		tray = null;
	}
}
