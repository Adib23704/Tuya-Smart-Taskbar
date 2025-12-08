/**
 * Electron Builder Configuration Script
 */

import builder from "electron-builder";

const Platform = builder.Platform;

async function build(): Promise<void> {
	try {
		await builder.build({
			targets: Platform.WINDOWS.createTarget("msi"),
			config: {
				appId: "adib.tuya.smart-taskbar",
				productName: "Tuya Smart Taskbar",
				directories: {
					output: "release",
				},
				files: ["dist/**/*", "pages/**/*", "assets/**/*", "package.json"],
				win: {
					target: ["msi"],
					icon: "assets/icon.ico",
					artifactName: "${productName}-${version}-setup.${ext}",
				},
				msi: {
					oneClick: false,
					createDesktopShortcut: true,
					createStartMenuShortcut: true,
				},
				extraResources: [
					{
						from: "assets/",
						to: "resources/assets",
					},
				],
				compression: "maximum",
				asar: true,
			},
		});

		console.log("Build completed successfully!");
	} catch (error) {
		console.error("Build failed:", error instanceof Error ? error.message : error);
		process.exit(1);
	}
}

build();
