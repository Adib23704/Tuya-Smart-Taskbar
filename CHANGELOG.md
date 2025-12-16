# Changelog

All notable changes to Tuya Smart Taskbar will be documented in this file.

## [2.0.1] - 2025-12-16

### Fixed

- **App freezing after toggling switches** - Fixed a critical deadlock caused by holding a read lock across async operations. The lock is now properly scoped and released before triggering menu updates.

- **Quit menu option not working** - Fixed incorrect memory ordering (`Relaxed` -> `Release`/`Acquire`) that prevented the shutdown flag from being visible across threads. Added a brief delay before exit to allow async tasks to terminate gracefully.

- **Context menu closing during auto-refresh** - Added interaction tracking that skips auto-refresh for 2 seconds after any menu interaction, preventing the menu from disappearing while in use.

- **Race conditions during concurrent menu updates** - Added a `MenuUpdateLock` mutex to serialize menu updates. Auto-refresh now uses a 100ms timeout to acquire the lock and skips if another update is in progress.

- **Hanging refresh operations** - Added a 15-second timeout to the auto-refresh operation to prevent slow network requests from blocking the refresh loop indefinitely.

### Changed

- **Auto-refresh interval increased from 5s to 10s** - Reduces contention and network traffic while still providing timely device status updates.

### Technical Details

- Refactored command handler to scope `SharedTuyaClient` lock properly
- Added `MENU_INTERACTION_TIME` atomic for tracking user interactions
- Added `MenuUpdateLock` type for serializing menu operations
- Changed all `Ordering::Relaxed` to proper `Release`/`Acquire` pairs for thread synchronization
- Wrapped auto-refresh in `tokio::time::timeout` for better error handling

## [2.0.0] - Initial Release

- System tray application for controlling Tuya smart home devices
- Support for switches, fans, air conditioners with temperature/mode/speed controls
- Auto-refresh device status every 10 seconds
- Configuration UI for API credentials
- Auto-launch on Windows startup option
- Single instance enforcement
