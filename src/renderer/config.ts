interface AppConfig {
	baseUrl: string;
	accessKey: string;
	secretKey: string;
	userId: string;
	runOnStartup: boolean;
}

((): void => {
	const baseUrlSelect = document.getElementById("baseUrl") as HTMLSelectElement;
	const accessKeyInput = document.getElementById("accessKey") as HTMLInputElement;
	const secretKeyInput = document.getElementById("secretKey") as HTMLInputElement;
	const userIdInput = document.getElementById("userId") as HTMLInputElement;
	const runOnStartupCheckbox = document.getElementById("runOnStartup") as HTMLInputElement;
	const saveButton = document.getElementById("save-btn") as HTMLButtonElement;

	function saveConfig(): void {
		const config: AppConfig = {
			baseUrl: baseUrlSelect.value,
			accessKey: accessKeyInput.value.trim(),
			secretKey: secretKeyInput.value.trim(),
			userId: userIdInput.value.trim(),
			runOnStartup: runOnStartupCheckbox.checked,
		};

		window.electronAPI.saveConfig(config);

		const originalText = saveButton.textContent;
		saveButton.textContent = "Saved!";
		saveButton.style.fontWeight = "bold";

		setTimeout(() => {
			saveButton.textContent = originalText;
			saveButton.style.fontWeight = "normal";
		}, 2000);
	}

	function loadConfig(config: AppConfig): void {
		baseUrlSelect.value = config.baseUrl || "https://openapi.tuyaeu.com";
		accessKeyInput.value = config.accessKey || "";
		secretKeyInput.value = config.secretKey || "";
		userIdInput.value = config.userId || "";
		runOnStartupCheckbox.checked = config.runOnStartup ?? true;
	}

	function init(): void {
		saveButton.addEventListener("click", saveConfig);

		window.electronAPI.onConfigData(loadConfig);
	}

	if (document.readyState === "loading") {
		document.addEventListener("DOMContentLoaded", init);
	} else {
		init();
	}
})();
