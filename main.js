import { app, Menu, Tray } from 'electron';
import { TuyaContext } from '@tuya/tuya-connector-nodejs';
import 'dotenv/config';
import process from 'process';

let tray = null;
let currentContextMenu = null;
let devices = [];

const tuya = new TuyaContext({
	baseUrl: process.env.TUYA_BASE_URL,
	accessKey: process.env.TUYA_ACCESS_KEY,
	secretKey: process.env.TUYA_SECRET_KEY,
});

const fetchDevices = async () => {
	try {
		const response = await tuya.request({
			method: 'GET',
			path: `/v1.0/users/${process.env.TUYA_USER_ID}/devices`,
		});
		return response.result;
	} catch (error) {
		console.error('Error fetching devices:', error);
		return [];
	}
};

const fetchDeviceStatus = async (deviceId) => {
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
};

const toggleDeviceState = async (deviceId, code, currentState) => {
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
};

const createDeviceMenu = (device, status) => {
	let statusItems = status.map((s) => {
		return {
			label: `${s.code} - ${s.value ? 'On' : 'Off'}`,
			click: async (_menuItem) => {
				await toggleDeviceState(device.id, s.code, s.value);
				updateMenu();
			},
			enabled: typeof s.value === 'boolean',
		};
	});

	statusItems = statusItems.filter((item) => item !== null);

	return {
		label: device.name,
		submenu: statusItems,
	};
};

const updateMenu = async () => {
	devices = await fetchDevices();
	const deviceMenuItems = await Promise.all(
		devices.map(async (device) => {
			const status = await fetchDeviceStatus(device.id);
			return createDeviceMenu(device, status);
		})
	);

	currentContextMenu = [
		...deviceMenuItems,
		{ type: 'separator' },
		{ label: 'Quit', role: 'quit' },
	];

	tray.setContextMenu(Menu.buildFromTemplate(currentContextMenu));
};

const startAutoRefresh = () => {
	setInterval(async () => {
		await updateMenu();
	}, 5000);
};

app.whenReady().then(async () => {
	tray = new Tray('./icon.ico');
	tray.setToolTip('Tuya Smart Device Control');

	await updateMenu();
	startAutoRefresh();
});