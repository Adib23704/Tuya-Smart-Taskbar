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
			'main.js',
			'html/**/*',
			'assets/**/*',
		],
		win: {
			target: ['msi'],
			icon: 'assets/icon.ico',
			artifactName: '${productName}-${version}-setup.${ext}',
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
		compression: 'maximum',
	}
}).then(() => {
	console.log('Build complete!');
}).catch((error) => {
	console.error('Error during build:', error.message || error);
});
