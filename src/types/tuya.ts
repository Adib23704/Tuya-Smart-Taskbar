export interface TuyaDevice {
	id: string;
	name: string;
	online: boolean;
	category: string;
	product_id: string;
	product_name: string;
	local_key: string;
	sub: boolean;
	uuid: string;
	owner_id: string;
	ip: string;
	time_zone: string;
	create_time: number;
	update_time: number;
	active_time: number;
	icon: string;
}

export interface TuyaDeviceStatus {
	code: string;
	value: boolean | string | number;
}

export interface TuyaCommand {
	code: string;
	value: boolean | string | number;
}

export interface TuyaCommandPayload {
	commands: TuyaCommand[];
}

export interface TuyaApiResponse<T> {
	success: boolean;
	result: T;
	t: number;
	tid: string;
}

export type DeviceStatusCode =
	| "switch"
	| "switch_1"
	| "switch_2"
	| "switch_led"
	| "fan_speed_percent"
	| "temp_set"
	| "windspeed"
	| "mode"
	| string;

export const AC_MODES = ["auto", "cold", "dry", "wind"] as const;
export type AcMode = (typeof AC_MODES)[number];

export const FAN_SPEED_LEVELS = [1, 2, 3, 4, 5] as const;
export const AC_FAN_SPEED_LEVELS = [1, 2, 3, 4] as const;
export const TEMPERATURE_RANGE = { min: 16, max: 30 } as const;
