import { TuyaContext } from "@tuya/tuya-connector-nodejs";
import type { AppConfig } from "../../types/config.js";

let tuyaContext: TuyaContext | null = null;

export function createTuyaContext(config: AppConfig): TuyaContext | null {
	if (config.baseUrl && config.accessKey && config.secretKey && config.userId) {
		tuyaContext = new TuyaContext({
			baseUrl: config.baseUrl,
			accessKey: config.accessKey,
			secretKey: config.secretKey,
		});
		return tuyaContext;
	}

	tuyaContext = null;
	return null;
}

export function getTuyaContext(): TuyaContext | null {
	return tuyaContext;
}

export function destroyTuyaContext(): void {
	tuyaContext = null;
}
