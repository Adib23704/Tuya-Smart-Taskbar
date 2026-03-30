# Changelog

All notable changes to Tuya Smart Taskbar will be documented in this file.

## [2.2.0] - 2026-03-31

### Added
- **Non-disruptive menu updates** - Auto-refresh now updates device states in-place without closing the open context menu. Check marks toggle in real-time while browsing the tray menu.
- **"Refresh Devices" menu item** - Manual refresh option in both the device menu and error menu, allowing users to retry after transient failures.
- **Offline device feedback** - Shows "No devices found" or "All N device(s) offline" in the tray menu when no online devices are available.
- **Linux (Xubuntu) support** - Full cross-platform support with Linux bundle targets (AppImage, Deb, RPM), setup script, and PNG tray icon. ([#1](https://github.com/Adib23704/Tuya-Smart-Taskbar/pull/1))
- **Biome linter/formatter** - Added `biome.json` configuration and lint/format/validate npm scripts for frontend code quality.
- **SVG accessibility** - All decorative SVGs now include `aria-hidden="true"` for accessibility compliance.

### Fixed
- **Update checker used string comparison instead of semver** - `commands/app.rs` now reuses the proper `is_newer_version()` from `update.rs` instead of naive `!=` comparison.
- **Update checker had no HTTP timeout** - Previously created `reqwest::Client::new()` with no timeout; now reuses the 10s-timeout client from `update.rs`.
- **Invalid `tar.gz` bundle target** - Removed from `tauri.conf.json` (not a valid Tauri v2 target), which was causing build failures.
- **Poisoned RwLock panics** - All `.unwrap()` calls on `RwLock` in `ConfigManager` replaced with `.unwrap_or_else(|p| p.into_inner())` to gracefully handle poisoned locks.
- **Atomic ordering too weak in token manager** - Changed `Ordering::Relaxed` to `Ordering::SeqCst` for `consecutive_failures` and `last_failure_time` atomics for proper cross-task synchronization.
- **Double read-lock on update state** - `update_tray_menu` previously acquired the `update_state` read lock twice in sequence; consolidated into a single acquisition.

### Changed
- **Menu item IDs are now stable** - Boolean toggles use `toggle:{deviceId}:{code}` format (handler reads current state from cache). Enum values use `set:{deviceId}:{code}:{value}`. This enables in-place updates without menu reconstruction.
- **Shared HTTP client** - `TuyaClient` and `TokenManager` now share a single `reqwest::Client` instance instead of creating separate ones.
- **Platform-neutral language** - Updated "Windows Taskbar App" to "System tray app" across `Cargo.toml`, `package.json`, `config.html`, and `about.html`.
- **Node.js version in setup script** - Updated from Node.js 18 (EOL) to Node.js 22 LTS.
- **Removed dead code** - Eliminated unused `should_check_for_update()`, `get_update_info()`, `CHECK_INTERVAL`, and `last_check` field from `update.rs`. Removed duplicate `UpdateInfo` struct and hardcoded URL from `commands/app.rs`.
- **Derived `PartialEq`** - `TuyaValue` and `TuyaDeviceStatus` now derive `PartialEq`, replacing ~35 lines of manual comparison code.

### Technical Details
- Introduced `MenuItemRegistry` (`Arc<RwLock<HashMap<String, CheckMenuItem>>>`) for in-place menu item state updates via `set_checked()`
- Added `is_structural_change()` to detect when device set or status codes change (triggers full rebuild) vs value-only changes (in-place update)
- Added `update_menu_items_in_place()` for real-time boolean, integer, and string status updates without menu reconstruction
- Toggle handler provides immediate visual feedback by updating registry + cache right after successful API call
- Clippy-clean: fixed `redundant_closure`, `collapsible_else_if`, `needless_borrows_for_generic_args`, `manual_is_multiple_of`
- Biome-clean: tab indentation, single quotes, `noDescendingSpecificity` disabled for inline HTML styles

## [2.1.0] - 2025-12-16

### Added
- **Automatic update notifications** - The app now proactively notifies users when a new version is available through multiple channels:
  - **Tray menu indicator** - "Update Available (vX.X.X)" menu item appears at the top of the tray menu when an update is detected
  - **Windows toast notification** - A system notification is displayed when a new update is first detected
  - **Tray icon change** - The tray icon changes to indicate an update is available
  - **Tooltip update** - The tray tooltip shows "Update Available" with the new version number
- **Background update checking** - Updates are checked automatically on startup (after 3 second delay) and periodically every hour
- New `update.rs` module for centralized update state management with proper version comparison

### Fixed
- **App freezing after toggling switches** - Fixed a critical deadlock caused by holding a read lock across async operations. The lock is now properly scoped and released before triggering menu updates.
- **Quit menu option not working** - Fixed incorrect memory ordering (`Relaxed` -> `Release`/`Acquire`) that prevented the shutdown flag from being visible across threads. Added a brief delay before exit to allow async tasks to terminate gracefully.
- **Context menu closing during auto-refresh** - Added interaction tracking that skips auto-refresh for 2 seconds after any menu interaction, preventing the menu from disappearing while in use.
- **Race conditions during concurrent menu updates** - Added a `MenuUpdateLock` mutex to serialize menu updates. Auto-refresh now uses a 100ms timeout to acquire the lock and skips if another update is in progress.
- **Hanging refresh operations** - Added a 15-second timeout to the auto-refresh operation to prevent slow network requests from blocking the refresh loop indefinitely.

### Changed
- **Auto-refresh interval increased from 5s to 10s** - Reduces contention and network traffic while still providing timely device status updates.

### Technical Details
- Added `tauri-plugin-notification` for Windows toast notifications
- Created `SharedUpdateState` for thread-safe update status tracking across the application
- Update checks use proper semantic version comparison (e.g., 2.0.1 < 2.0.2 < 2.1.0)
- Notifications are shown only once per update detection to avoid notification spam
- Added `update.ico` tray icon for visual update indication
- Refactored command handler to scope `SharedTuyaClient` lock properly
- Added `MENU_INTERACTION_TIME` atomic for tracking user interactions
- Added `MenuUpdateLock` type for serializing menu operations
- Changed all `Ordering::Relaxed` to proper `Release`/`Acquire` pairs for thread synchronization
- Wrapped auto-refresh in `tokio::time::timeout` for better error handling

## [2.0.0] - Initial Release
**A complete rewrite - now faster, lighter, and more reliable!**
This major release represents a ground-up rebuild of Tuya Smart Taskbar, migrating from Electron to Tauri v2 with a native Rust backend. The result is a dramatically smaller, faster, and more resource-efficient application.

## Highlights
- **~95% smaller installer** - From ~90MB to ~5MB
- **Native performance** - Rust backend with minimal memory footprint
- **Improved reliability** - Automatic token refresh and smart error recovery
- **Better UX** - Redesigned configuration page with real-time validation

## Changelog
### Added
- Native Rust backend using Tauri v2 framework
- HMAC-SHA256 authentication for Tuya Cloud API
- Automatic token management with refresh 5 minutes before expiry
- Rate limiting protection (max 5 consecutive failures, 60s cooldown)
- HTTP retry logic with exponential backoff (3 retries, 500ms starting delay)
- Device status caching to prevent unnecessary tray menu rebuilds
- Dynamic system tray menus with device controls:
  - Toggle switches (on/off)
  - Fan speed control
  - Temperature adjustment
  - AC mode selection
- Redesigned configuration page with:
  - Modern Inter font and improved styling
  - Real-time form validation with visual feedback
  - Password visibility toggle for secret key
  - Loading indicators and status messages
- Embedded tray icons for better reliability
- Comprehensive README with installation and usage instructions

### Changed
- Complete migration from Electron/TypeScript to Tauri v2/Rust
- Frontend moved to `/frontend` directory
- Configuration now stored in `%LOCALAPPDATA%/Tuya Smart Taskbar/config.json`
- Config path retrieval now uses BaseDirs for better cross-platform compatibility
- Logo image source updated from PNG to ICO format

### Removed
- All Electron and TypeScript backend code
- Unused methods in ConfigManager, TuyaClient, and TokenManager

### Fixed
- Tray icon loading now uses embedded byte arrays instead of file paths
- Password visibility toggle functionality improved
- Various layout adjustments for better spacing and alignment
