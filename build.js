import builder from 'electron-builder';
import process from 'process';

const { Platform } = builder;

const args = process.argv.slice(2);
const buildAll = args.includes('--all');
const buildTargets = {
	windows: args.includes('--windows') || buildAll,
	mac: args.includes('--macos') || buildAll,
	linux: args.includes('--linux') || buildAll,
};

const selectedTargets = [];

if (buildTargets.windows) selectedTargets.push(Platform.WINDOWS.createTarget());
if (buildTargets.mac) selectedTargets.push(Platform.MAC.createTarget());
if (buildTargets.linux) selectedTargets.push(Platform.LINUX.createTarget());

if (selectedTargets.length === 0) {
	selectedTargets.push(Platform.current().createTarget());
}

const config = {
	directories: {
		output: 'dist',
		buildResources: 'assets',
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
		icon: 'assets/icon.ico',
	},
	mac: {
		target: ['dmg', 'zip'],
		icon: 'assets/icon.icns',
	},
	linux: {
		target: ['AppImage', 'deb'],
		icon: 'assets/icon.png',
		category: 'Utility',
	},
	nsis: {
		oneClick: false,
		allowToChangeInstallationDirectory: true,
		createDesktopShortcut: true,
		createStartMenuShortcut: true,
	},
	msi: {
		oneClick: false,
		createDesktopShortcut: true,
		createStartMenuShortcut: true,
	}
};

Promise.all(
	selectedTargets.map((target) =>
		builder.build({
			targets: target,
			config: config,
		})
	)
)
	.then(() => {
		console.log('Build complete for all specified platforms!');
	})
	.catch((error) => {
		console.error('Error during build:', error);
		process.exit(1);
	});
