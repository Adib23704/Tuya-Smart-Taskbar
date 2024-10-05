import { app, Menu, Tray, BrowserWindow, ipcMain } from 'electron';
import { TuyaContext } from '@tuya/tuya-connector-nodejs';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

let tray = null;
let currentContextMenu = null;
let configWindow = null;
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
	};
}

function saveConfig(config) {
	fs.writeFileSync(configPath, JSON.stringify(config));
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
					value: !currentState,
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
	const statusItems = status.map((s) => ({
		label: `${(s.code.charAt(0).toUpperCase() + s.code.slice(1)).replace(/_/g, ' ')} - ${s.value ? 'On' : 'Off'}`,
		click: async () => {
			await toggleDeviceState(device.id, s.code, s.value);
			updateMenu();
		},
		enabled: typeof s.value === 'boolean',
	}));

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
		height: 500,
		resizable: false,
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		frame: false,
		title: 'Tuya Configurations',
		icon: './icon.ico',
		autoHideMenuBar: true,
	});

	configWindow.loadFile('config.html');

	configWindow.on('closed', () => {
		configWindow = null;
	});

	configWindow.webContents.on('did-finish-load', () => {
		configWindow.webContents.send('config-data', config);
	});
}

app.whenReady().then(() => {
	tray = new Tray(defaultIconPath);
	tray.setToolTip('Tuya Smart Device Control');

	config = loadConfig();
	tuya = createTuyaContext();
	updateMenu();
	startAutoRefresh();

	ipcMain.on('save-config', (event, newConfig) => {
		config = newConfig;
		saveConfig(config);
		tuya = createTuyaContext();
		updateMenu();
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
