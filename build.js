import builder from 'electron-builder';
const Platform = builder.Platform;

builder.build({
	targets: Platform.WINDOWS.createTarget('msi'),
	config: {
		appId: 'lol.adib.tuya-smart-taskbar',
		productName: 'Tuya Smart Taskbar',
		directories: {
			output: 'dist',
		},
		files: [
			'main.js',             // Your main application code
			'html/**/*',            // HTML folder for config.html
			'assets/**/*',          // Assets (icons, etc.)
		],
		win: {
			target: ['msi'],
			icon: 'assets/icon.ico',  // The main icon for the installer
			artifactName: '${productName}-${version}-setup.${ext}',  // Naming format
		},
		msi: {
			oneClick: false,
			createDesktopShortcut: true,
			createStartMenuShortcut: true,
		},
		extraResources: {
			from: 'assets/',
			to: 'resources/assets'
		},
		compression: 'store',
	}
}).then(() => {
	console.log('Build complete!');
}).catch((error) => {
	console.error('Error during build:', error.message || error);
});
