import type { TuyaCommandPayload, TuyaDevice, TuyaDeviceStatus } from "../../types/tuya.js";
import { getTuyaContext } from "./context.js";

export async function fetchDevices(userId: string): Promise<TuyaDevice[]> {
	const tuya = getTuyaContext();
	if (!tuya) {
		return [];
	}

	try {
		const response = await tuya.request({
			method: "GET",
			path: `/v1.0/users/${userId}/devices`,
		});
		return (response.result as TuyaDevice[]) || [];
	} catch (error) {
		console.error("Error fetching devices:", error);
		return [];
	}
}

export async function fetchDeviceStatus(deviceId: string): Promise<TuyaDeviceStatus[]> {
	const tuya = getTuyaContext();
	if (!tuya) {
		return [];
	}

	try {
		const response = await tuya.request({
			method: "GET",
			path: `/v1.0/devices/${deviceId}/status`,
		});

		return (response.result as TuyaDeviceStatus[]) || [];
	} catch (error) {
		console.error("Error fetching device status:", error);
		return [];
	}
}

export async function sendDeviceCommand(
	deviceId: string,
	code: string,
	value: boolean | string | number,
): Promise<boolean> {
	const tuya = getTuyaContext();
	if (!tuya) {
		return false;
	}

	try {
		const command: TuyaCommandPayload = {
			commands: [{ code, value }],
		};

		await tuya.request({
			method: "POST",
			path: `/v1.0/devices/${deviceId}/commands`,
			body: command,
		});

		return true;
	} catch (error) {
		console.error("Error sending device command:", error);
		return false;
	}
}

export async function toggleDeviceState(
	deviceId: string,
	code: string,
	currentValue: boolean | string | number,
): Promise<boolean> {
	const newValue = typeof currentValue === "boolean" ? !currentValue : currentValue;

	return await sendDeviceCommand(deviceId, code, newValue);
}
