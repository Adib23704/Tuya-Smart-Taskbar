import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { app } from "electron";
import { type AppConfig, DEFAULT_CONFIG } from "../types/config.js";

const CONFIG_FILENAME = "config.json";

class ConfigManager {
	private configPath: string;
	private config: AppConfig;

	constructor() {
		this.configPath = join(app.getPath("userData"), CONFIG_FILENAME);
		this.config = this.load();
	}

	private load(): AppConfig {
		try {
			if (existsSync(this.configPath)) {
				const data = readFileSync(this.configPath, "utf-8");
				const parsed = JSON.parse(data) as Partial<AppConfig>;

				return {
					...DEFAULT_CONFIG,
					...parsed,
				};
			}
		} catch (error) {
			console.error("Error loading config:", error);
		}

		return { ...DEFAULT_CONFIG };
	}

	save(config: AppConfig): void {
		try {
			this.config = config;
			writeFileSync(this.configPath, JSON.stringify(config, null, 2), "utf-8");
		} catch (error) {
			console.error("Error saving config:", error);
			throw new Error("Failed to save configuration");
		}
	}

	get(): AppConfig {
		return { ...this.config };
	}

	update(partial: Partial<AppConfig>): void {
		this.save({
			...this.config,
			...partial,
		});
	}

	isConfigured(): boolean {
		return !!(
			this.config.baseUrl &&
			this.config.accessKey &&
			this.config.secretKey &&
			this.config.userId
		);
	}

	updateStartupSettings(): void {
		app.setLoginItemSettings({
			openAtLogin: this.config.runOnStartup,
			openAsHidden: this.config.runOnStartup,
		});
	}
}

let configManagerInstance: ConfigManager | null = null;

export function getConfigManager(): ConfigManager {
	if (!configManagerInstance) {
		configManagerInstance = new ConfigManager();
	}
	return configManagerInstance;
}

export function initConfigManager(): ConfigManager {
	return getConfigManager();
}
