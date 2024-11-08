import { app, Menu, Tray, BrowserWindow, ipcMain } from 'electron';
import { TuyaContext } from '@tuya/tuya-connector-nodejs';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import fetch from 'node-fetch';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const appVersion = app.getVersion();
const downloadUrl = 'https://github.com/Adib23704/Tuya-Smart-Taskbar/releases/latest';

let tray = null;
let currentContextMenu = null;
let configWindow = null;
let aboutWindow = null;
let devices = [];
let config;
let tuya;

const configPath = path.join(app.getPath('userData'), 'config.json');
const defaultIconPath = path.join(__dirname, 'assets/icon.ico');
const loadingIconPath = path.join(__dirname, 'assets/loading.ico');

function loadConfig() {
	if (fs.existsSync(configPath)) {
		const configFile = fs.readFileSync(configPath);
		return JSON.parse(configFile);
	}
	return {
		baseUrl: '',
		accessKey: '',
		secretKey: '',
		userId: '',
		runOnStartup: true,
	};
}

function saveConfig(config) {
	fs.writeFileSync(configPath, JSON.stringify(config));
}

function updateStartupSettings(runOnStartup) {
	app.setLoginItemSettings({
		openAtLogin: runOnStartup,
		openAsHidden: runOnStartup
	});
}

function createTuyaContext() {
	if (config.baseUrl && config.accessKey && config.secretKey && config.userId) {
		return new TuyaContext({
			baseUrl: config.baseUrl,
			accessKey: config.accessKey,
			secretKey: config.secretKey,
		});
	}

	return null;
};

function setTrayIconLoading(isLoading) {
	if (isLoading) {
		tray.setImage(loadingIconPath);
	} else {
		tray.setImage(defaultIconPath);
	}
}

async function fetchDevices() {
	if (!tuya) return;
	try {
		const response = await tuya.request({
			method: 'GET',
			path: `/v1.0/users/${config.userId}/devices`,
		});
		return response.result;
	} catch (error) {
		console.error('Error fetching devices:', error);
		return [];
	}
}

async function fetchDeviceStatus(deviceId) {
	if (!tuya) return;
	try {
		const response = await tuya.request({
			method: 'GET',
			path: `/v1.0/devices/${deviceId}/status`,
		});
		return response.result;
	} catch (error) {
		console.error('Error fetching device status:', error);
		return [];
	}
}

async function toggleDeviceState(deviceId, code, currentState) {
	if (!tuya) return;
	try {
		const command = {
			commands: [
				{
					code,
					value: (typeof currentState === 'boolean') ? !currentState : currentState,
				},
			],
		};
		await tuya.request({
			method: 'POST',
			path: `/v1.0/devices/${deviceId}/commands`,
			body: command,
		});
	} catch (error) {
		console.error('Error toggling device state:', error);
	}
}

function createDeviceMenu(device, status) {
	let statusItems = status.map((s) => {
		if (typeof s.value === 'boolean') {
			return {
				label: `${(s.code.charAt(0).toUpperCase() + s.code.slice(1)).replace(/_/g, ' ')}`,
				click: async () => {
					await toggleDeviceState(device.id, s.code, s.value);
					updateMenu();
				},
				enabled: true,
				type: 'checkbox',
				checked: s.value,
			};
		}

		if (s.code === 'fan_speed_percent') {
			s.value = parseInt(s.value, 10);
			return {
				label: 'Fan Speed',
				submenu: Array.from({ length: 5 }, (_, i) => ({
					label: `${i + 1}`,
					click: async () => {
						await toggleDeviceState(device.id, s.code, (i + 1).toString());
						updateMenu();
					},
					type: 'checkbox',
					checked: s.value === i + 1,
				})),
			};
		} else if (s.code === 'temp_set') {
			return {
				label: 'Temperature',
				submenu: Array.from({ length: 15 }, (_, i) => ({
					label: `${i + 16}`,
					click: async () => {
						await toggleDeviceState(device.id, s.code, i + 16);
						updateMenu();
					},
					type: 'checkbox',
					checked: s.value === i + 16,
				})),
			};
		} else if (s.code === 'windspeed') {
			s.value = parseInt(s.value, 10);
			return {
				label: 'AC Fan Speed',
				submenu: Array.from({ length: 4 }, (_, i) => ({
					label: `${i + 1}`,
					click: async () => {
						await toggleDeviceState(device.id, s.code, (i + 1).toString());
						updateMenu();
					},
					type: 'checkbox',
					checked: s.value === i + 1,
				})),
			};
		} else if (s.code === 'mode') {
			return {
				label: 'AC Mode',
				submenu: ['auto', 'cold', 'dry', 'wind'].map((mode) => ({
					label: mode.charAt(0).toUpperCase() + mode.slice(1),
					click: async () => {
						await toggleDeviceState(device.id, s.code, mode);
						updateMenu();
					},
					type: 'checkbox',
					checked: s.value === mode,
				})),
			};
		}

		return null;
	});

	statusItems = statusItems.filter((item) => item !== null);

	return {
		label: device.name,
		submenu: statusItems,
	};
}

async function updateMenu(auto = false) {
	if (!tuya) {
		currentContextMenu = [
			{
				label: 'Open Configuration',
				click: openConfigWindow,
			},
			{ label: 'Quit', role: 'quit' },
		];
	} else {
		if (!auto) setTrayIconLoading(true);
		devices = await fetchDevices();
		let deviceMenuItems = await Promise.all(
			devices.map(async (device) => {
				if (device.online) {
					const status = await fetchDeviceStatus(device.id);
					return createDeviceMenu(device, status);
				}
				return null;
			})
		);

		deviceMenuItems = deviceMenuItems.filter((item) => item !== null);

		currentContextMenu = [
			...deviceMenuItems,
			{ type: 'separator' },
			{
				label: 'Open Configuration',
				click: openConfigWindow,
			},
			{
				label: 'About',
				click: openAboutWindow,
			},
			{ label: 'Quit', role: 'quit' },
		];
	}

	tray.setContextMenu(Menu.buildFromTemplate(currentContextMenu));
	if (!auto) setTrayIconLoading(false);
};

function openConfigWindow() {
	if (configWindow) {
		configWindow.focus();
		return;
	}

	configWindow = new BrowserWindow({
		width: 400,
		height: 580,
		resizable: false,
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		title: 'Tuya Smart Taskbar Config',
		icon: defaultIconPath,
		autoHideMenuBar: true,
		center: true,
		fullscreenable: false,
		movable: true
	});

	configWindow.loadFile('html/config.html');

	configWindow.on('closed', () => {
		configWindow = null;
	});

	configWindow.webContents.on('did-finish-load', () => {
		configWindow.webContents.send('config-data', config);
	});
}

function openAboutWindow() {
	if (aboutWindow) {
		aboutWindow.focus();
		return;
	}

	aboutWindow = new BrowserWindow({
		width: 400,
		height: 500,
		resizable: false,
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		title: 'About Tuya Smart Taskbar',
		icon: defaultIconPath,
		autoHideMenuBar: true,
		center: true,
		fullscreenable: false,
		movable: true
	});

	aboutWindow.loadFile('html/about.html');

	aboutWindow.on('closed', () => {
		aboutWindow = null;
	});

	aboutWindow.webContents.on('did-finish-load', () => {
		aboutWindow.webContents.send('about-data', appVersion);
	});
}

app.whenReady().then(() => {
	tray = new Tray(defaultIconPath);
	tray.setToolTip('Tuya Smart Taskbar');

	config = loadConfig();
	updateStartupSettings(config.runOnStartup);
	tuya = createTuyaContext();
	updateMenu();
	startAutoRefresh();

	ipcMain.on('save-config', (event, newConfig) => {
		config = newConfig;
		saveConfig(config);
		updateStartupSettings(config.runOnStartup);
		tuya = createTuyaContext();
		updateMenu();
	});

	ipcMain.on('check-for-update', async (event) => {
		try {
			const response = await fetch(
				'https://raw.githubusercontent.com/Adib23704/Tuya-Smart-Taskbar/refs/heads/master/package.json'
			);
			const data = await response.json();
			const latestVersion = data.version;
			console.log('Latest version:', latestVersion);
			if (latestVersion !== appVersion) {
				event.sender.send('update-available', true, latestVersion, downloadUrl);
			} else {
				event.sender.send('update-available', false);
			}
		} catch (error) {
			console.error('Error checking for update:', error);
			event.sender.send('update-check-failed');
		}
	});

	app.on('window-all-closed', (event) => {
		event.preventDefault();
	});
});

function startAutoRefresh() {
	setInterval(async () => {
		await updateMenu(true);
	}, 5000);
}
