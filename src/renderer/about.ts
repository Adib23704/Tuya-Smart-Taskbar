((): void => {
	const appVersionElement = document.getElementById("appVersion") as HTMLElement;
	const checkUpdateButton = document.getElementById("check-update-btn") as HTMLButtonElement;

	const BUTTON_STATES = {
		DEFAULT: {
			text: "Check for Update",
			disabled: false,
			backgroundColor: "#eb312a",
		},
		CHECKING: {
			text: "Checking...",
			disabled: true,
			backgroundColor: "#999",
		},
		UP_TO_DATE: {
			text: "You're on the latest version!",
			disabled: true,
			backgroundColor: "#4CAF50",
		},
		ERROR: {
			text: "Check failed. Try again.",
			disabled: false,
			backgroundColor: "#eb312a",
		},
	} as const;

	type ButtonState = keyof typeof BUTTON_STATES;

	function setButtonState(state: ButtonState): void {
		const { text, disabled, backgroundColor } = BUTTON_STATES[state];
		checkUpdateButton.textContent = text;
		checkUpdateButton.disabled = disabled;
		checkUpdateButton.style.backgroundColor = backgroundColor;
	}

	function checkForUpdate(): void {
		setButtonState("CHECKING");
		window.electronAPI.checkForUpdate();
	}

	function handleUpdateAvailable(available: boolean, version?: string, downloadUrl?: string): void {
		if (available && version && downloadUrl) {
			const shouldUpdate = window.confirm(
				`A new version (${version}) is available. Would you like to download it?`,
			);

			if (shouldUpdate) {
				window.electronAPI.openExternal(downloadUrl);
			}

			setButtonState("DEFAULT");
		} else {
			setButtonState("UP_TO_DATE");

			setTimeout(() => {
				setButtonState("DEFAULT");
			}, 3000);
		}
	}

	function handleUpdateCheckFailed(): void {
		setButtonState("ERROR");

		setTimeout(() => {
			setButtonState("DEFAULT");
		}, 3000);
	}

	function openExternalLink(url: string): void {
		window.electronAPI.openExternal(url);
	}

	(window as unknown as Record<string, unknown>).openExternalLink = openExternalLink;

	function loadVersion(version: string): void {
		appVersionElement.textContent = version;
	}

	function init(): void {
		checkUpdateButton.addEventListener("click", checkForUpdate);

		window.electronAPI.onAboutData(loadVersion);
		window.electronAPI.onUpdateAvailable(handleUpdateAvailable);
		window.electronAPI.onUpdateCheckFailed(handleUpdateCheckFailed);
	}

	if (document.readyState === "loading") {
		document.addEventListener("DOMContentLoaded", init);
	} else {
		init();
	}
})();
