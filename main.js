import { app, Menu, Tray, BrowserWindow, ipcMain } from 'electron';
import { TuyaContext } from '@tuya/tuya-connector-nodejs';
import fs from 'fs';
import path from 'path';

let tray = null;
let configWindow = null;
let devices = [];
let config;
let tuya;

const configPath = path.join(app.getPath('userData'), 'config.json');

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
	return new TuyaContext({
		baseUrl: config.baseUrl,
		accessKey: config.accessKey,
		secretKey: config.secretKey,
	});
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
		label: `${s.code} - ${s.value ? 'On' : 'Off'}`,
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

async function updateMenu() {
	devices = await fetchDevices();
	const deviceMenuItems = await Promise.all(
		devices.map(async (device) => {
			const status = await fetchDeviceStatus(device.id);
			return createDeviceMenu(device, status);
		})
	);

	const contextMenu = Menu.buildFromTemplate([
		...deviceMenuItems,
		{ type: 'separator' },
		{
			label: 'Open Configuration',
			click: openConfigWindow,
		},
		{ label: 'Quit', role: 'quit' },
	]);

	tray.setContextMenu(contextMenu);
}

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
	tray = new Tray('./icon.ico');
	tray.setToolTip('Tuya Smart Device Control');

	config = loadConfig();
	if (!config.baseUrl || !config.accessKey || !config.secretKey || !config.userId) {
		openConfigWindow();
	} else {
		tuya = createTuyaContext();
		updateMenu();
		startAutoRefresh();
	}

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
		await updateMenu();
	}, 5000);
}
