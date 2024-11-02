import builder from 'electron-builder';
import process from 'process';

const Platform = builder.Platform;

const args = process.argv.slice(2);
const buildWindows = args.includes('--windows');
const buildMacOS = args.includes('--macos');
const buildLinux = args.includes('--linux');
const buildAll = args.includes('--all') || (buildWindows && buildMacOS && buildLinux);

let targets = [];

if (buildAll) {
	targets = [
		Platform.WINDOWS.createTarget(),
		Platform.MAC.createTarget(),
		Platform.LINUX.createTarget()
	];
} else {
	if (buildWindows) {
		targets.push(Platform.WINDOWS.createTarget());
	}
	if (buildMacOS) {
		targets.push(Platform.MAC.createTarget());
	}
	if (buildLinux) {
		targets.push(Platform.LINUX.createTarget());
	}
	if (targets.length === 0) {
		targets.push(Platform.current().createTarget());
	}
}

const config = {
	directories: {
		output: 'dist',
		buildResources: 'assets'
	},
	files: [
		'**/*',
		'!**/node_modules/*/{CHANGELOG.md,README.md,README,readme.md,readme}',
		'!**/node_modules/*/{test,__tests__,tests,powered-test,example,examples}',
		'!**/node_modules/*.d.ts',
		'!**/node_modules/.bin',
		'!**/*.{iml,o,hprof,orig,pyc,pyo,rbc,swp,csproj,sln,xproj}',
		'!.editorconfig',
		'!**/._*',
		'!**/{.DS_Store,.git,.hg,.svn,CVS,RCS,SCCS,.gitignore,.gitattributes}',
		'!**/{__pycache__,thumbs.db,.flowconfig,.idea,.vs,.nyc_output}',
		'!**/{appveyor.yml,.travis.yml,circle.yml}',
		'!**/{npm-debug.log,yarn.lock,.yarn-integrity,.yarn-metadata.json}',
	],
	appId: 'lol.adib.tuya-smart-taskbar',
	productName: 'Tuya Smart Taskbar',
	asar: true,
	win: {
		target: ['nsis', 'msi'],
		icon: 'assets/icon.ico'
	},
	mac: {
		target: ['dmg'],
		icon: 'assets/icon.icns'
	},
	linux: {
		target: ['AppImage', 'deb'],
		icon: 'assets/icon.png',
		category: 'Utility'
	},
	nsis: {
		oneClick: false,
		allowToChangeInstallationDirectory: true,
		createDesktopShortcut: true,
		createStartMenuShortcut: true
	},
	msi: {
		oneClick: false
	}
};

Promise.all(targets.map(target => builder.build({
	targets: target,
	config: config
})))
	.then(() => {
		console.log('Build complete for all specified platforms!');
	})
	.catch((error) => {
		console.error('Error during build:', error);
		process.exit(1);
	});