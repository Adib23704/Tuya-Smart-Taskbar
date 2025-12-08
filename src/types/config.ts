export interface AppConfig {
	baseUrl: string;
	accessKey: string;
	secretKey: string;
	userId: string;
	runOnStartup: boolean;
}

export const DEFAULT_CONFIG: AppConfig = {
	baseUrl: "",
	accessKey: "",
	secretKey: "",
	userId: "",
	runOnStartup: true,
};

export const TUYA_REGIONS = {
	CENTRAL_EUROPE: "https://openapi.tuyaeu.com",
	CHINA: "https://openapi.tuyacn.com",
	WESTERN_AMERICA: "https://openapi.tuyaus.com",
	EASTERN_AMERICA: "https://openapi-ueaz.tuyaus.com",
	WESTERN_EUROPE: "https://openapi-weaz.tuyaeu.com",
	INDIA: "https://openapi.tuyain.com",
} as const;

export type TuyaRegion = (typeof TUYA_REGIONS)[keyof typeof TUYA_REGIONS];
