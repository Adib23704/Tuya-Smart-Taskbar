#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{
    image::Image,
    menu::MenuEvent,
    tray::TrayIconBuilder,
    AppHandle, Manager, RunEvent, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::{Mutex, RwLock};

use tuya_smart_taskbar::{
    commands,
    config::{set_auto_launch, ConfigManager},
    tray, tuya,
    tuya::{create_shared_client, initialize_client, SharedTuyaClient, TuyaDeviceStatus},
    update::{self, create_update_state, SharedUpdateState},
};

static RUNNING: AtomicBool = AtomicBool::new(false);
static MENU_INTERACTION_TIME: AtomicI64 = AtomicI64::new(0);
static UPDATE_CHECK_COUNTER: AtomicU64 = AtomicU64::new(0);

type DeviceStatusCache = Arc<RwLock<HashMap<String, Vec<TuyaDeviceStatus>>>>;
type MenuUpdateLock = Arc<Mutex<()>>;

const ICON_BYTES: &[u8] = include_bytes!("../icons/icon.ico");
const LOADING_ICON_BYTES: &[u8] = include_bytes!("../icons/loading.ico");
const UPDATE_ICON_BYTES: &[u8] = include_bytes!("../icons/update.ico");
const UPDATE_CHECK_INTERVAL: u64 = 360;

async fn update_tray_menu(
    app: &AppHandle,
    is_auto_refresh: bool,
    status_cache: &DeviceStatusCache,
    menu_lock: &MenuUpdateLock,
    update_state: &SharedUpdateState,
) {
    if is_auto_refresh {
        let last_interaction = MENU_INTERACTION_TIME.load(Ordering::SeqCst);
        let now = chrono::Utc::now().timestamp_millis();
        if now - last_interaction < 2000 {
            tracing::debug!("Skipping auto-refresh: recent menu interaction");
            return;
        }
    }

    let _guard = if is_auto_refresh {
        match tokio::time::timeout(Duration::from_millis(100), menu_lock.lock()).await {
            Ok(guard) => guard,
            Err(_) => {
                tracing::debug!("Skipping auto-refresh: menu update in progress");
                return;
            }
        }
    } else {
        menu_lock.lock().await
    };

    let config_manager = app.state::<ConfigManager>();
    let client = app.state::<SharedTuyaClient>();

    if !is_auto_refresh {
        if let Some(tray) = app.tray_by_id("main") {
            if let Ok(icon) = Image::from_bytes(LOADING_ICON_BYTES) {
                let _ = tray.set_icon(Some(icon));
            }
        }
    }

    let (menu, new_cache) = if !config_manager.is_configured() {
        (tray::build_unconfigured_menu(app, update_state).await, None)
    } else {
        match tray::build_device_menu_with_cache(app, &client, &config_manager, update_state).await
        {
            Ok((menu, device_statuses)) => (Ok(menu), Some(device_statuses)),
            Err(e) => {
                tracing::error!("Error building device menu: {}", e);
                (tray::build_error_menu(app, update_state).await, None)
            }
        }
    };

    if !is_auto_refresh {
        let has_update = {
            let guard = update_state.read().await;
            guard.update_available
        };
        if let Some(tray) = app.tray_by_id("main") {
            let icon_bytes = if has_update {
                UPDATE_ICON_BYTES
            } else {
                ICON_BYTES
            };
            if let Ok(icon) = Image::from_bytes(icon_bytes) {
                let _ = tray.set_icon(Some(icon));
            }
            let tooltip = if has_update {
                let guard = update_state.read().await;
                if let Some(ref version) = guard.latest_version {
                    format!("Tuya Smart Taskbar - Update Available (v{})", version)
                } else {
                    "Tuya Smart Taskbar - Update Available".to_string()
                }
            } else {
                "Tuya Smart Taskbar".to_string()
            };
            let _ = tray.set_tooltip(Some(&tooltip));
        }
    }

    let should_update_menu = if is_auto_refresh {
        if let Some(ref new_statuses) = new_cache {
            let cache = status_cache.read().await;
            let has_changes = !statuses_equal(&cache, new_statuses);
            if has_changes {
                tracing::debug!("Device states changed, updating menu");
            }
            has_changes
        } else {
            true
        }
    } else {
        true
    };

    if let Some(new_statuses) = new_cache {
        let mut cache = status_cache.write().await;
        *cache = new_statuses;
    }

    if should_update_menu {
        if let Ok(menu) = menu {
            if let Some(tray) = app.tray_by_id("main") {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }
}

fn statuses_equal(
    old: &HashMap<String, Vec<TuyaDeviceStatus>>,
    new: &HashMap<String, Vec<TuyaDeviceStatus>>,
) -> bool {
    if old.len() != new.len() {
        return false;
    }
    for (device_id, old_statuses) in old {
        match new.get(device_id) {
            None => return false,
            Some(new_statuses) => {
                if old_statuses.len() != new_statuses.len() {
                    return false;
                }
                for (old_s, new_s) in old_statuses.iter().zip(new_statuses.iter()) {
                    if old_s.code != new_s.code || !values_equal(&old_s.value, &new_s.value) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn values_equal(a: &tuya::TuyaValue, b: &tuya::TuyaValue) -> bool {
    match (a, b) {
        (tuya::TuyaValue::Boolean(a), tuya::TuyaValue::Boolean(b)) => a == b,
        (tuya::TuyaValue::String(a), tuya::TuyaValue::String(b)) => a == b,
        (tuya::TuyaValue::Integer(a), tuya::TuyaValue::Integer(b)) => a == b,
        (tuya::TuyaValue::Float(a), tuya::TuyaValue::Float(b)) => (a - b).abs() < f64::EPSILON,
        _ => false,
    }
}

async fn check_and_notify_update(
    app: &AppHandle,
    update_state: &SharedUpdateState,
    status_cache: Option<&DeviceStatusCache>,
    menu_lock: Option<&MenuUpdateLock>,
) -> bool {
    tracing::debug!("Checking for updates...");

    if let Some(update_info) = update::check_for_update(app).await {
        tracing::debug!(
            "Update check result: current={}, latest={}, available={}",
            update_info.current_version,
            update_info.latest_version,
            update_info.available
        );

        let (is_new, should_notify) = update::update_state(update_state, &update_info).await;

        if is_new {
            tracing::info!(
                "Update available: {} -> {}",
                update_info.current_version,
                update_info.latest_version
            );

            if let Some(tray) = app.tray_by_id("main") {
                if let Ok(icon) = Image::from_bytes(UPDATE_ICON_BYTES) {
                    let _ = tray.set_icon(Some(icon));
                }
                let tooltip = format!(
                    "Tuya Smart Taskbar - Update Available (v{})",
                    update_info.latest_version
                );
                let _ = tray.set_tooltip(Some(&tooltip));
            }

            if let (Some(cache), Some(lock)) = (status_cache, menu_lock) {
                tracing::debug!("Rebuilding menu to show update indicator");
                update_tray_menu(app, false, cache, lock, update_state).await;
            }
        }

        if should_notify {
            tracing::info!("Sending update notification");
            match app
                .notification()
                .builder()
                .title("Tuya Smart Taskbar Update Available")
                .body(format!(
                    "Version {} is now available. Click the tray icon to update.",
                    update_info.latest_version
                ))
                .show()
            {
                Ok(_) => tracing::debug!("Notification sent successfully"),
                Err(e) => tracing::error!("Failed to send notification: {}", e),
            }
        }

        return is_new;
    } else {
        tracing::debug!("Update check failed or returned no info");
    }

    false
}

fn handle_menu_event(
    app: &AppHandle,
    event: MenuEvent,
    status_cache: DeviceStatusCache,
    menu_lock: MenuUpdateLock,
    update_state: SharedUpdateState,
) {
    MENU_INTERACTION_TIME.store(chrono::Utc::now().timestamp_millis(), Ordering::SeqCst);

    let id = event.id().as_ref();
    tracing::debug!("Menu event: {}", id);

    match id {
        "open_config" => {
            open_config_window(app);
        }
        "open_about" => {
            open_about_window(app);
        }
        "open_update" => {
            let _ = open::that(update::get_download_url());
        }
        "quit" => {
            RUNNING.store(false, Ordering::Release);
            std::thread::sleep(Duration::from_millis(100));
            app.exit(0);
        }
        _ if id.starts_with("cmd:") => {
            if let Some((device_id, code, value_str)) = tray::parse_command_id(id) {
                let value = tray::parse_value(&value_str);
                let app_handle = app.clone();
                let cache = status_cache.clone();
                let lock = menu_lock.clone();
                let update_st = update_state.clone();

                tauri::async_runtime::spawn(async move {
                    let result = {
                        let client = app_handle.state::<SharedTuyaClient>();
                        let guard = client.read().await;
                        if let Some(tuya_client) = guard.as_ref() {
                            Some(
                                tuya_client
                                    .send_device_command(&device_id, &code, value)
                                    .await,
                            )
                        } else {
                            None
                        }
                    };

                    match result {
                        Some(Ok(_)) => {
                            tracing::info!("Command sent: {}:{}", code, value_str);
                            update_tray_menu(&app_handle, false, &cache, &lock, &update_st).await;
                        }
                        Some(Err(e)) => {
                            tracing::error!("Failed to send command: {}", e);
                        }
                        None => {
                            tracing::error!("Client not initialized");
                        }
                    }
                });
            }
        }
        _ => {}
    }
}

fn open_config_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("config") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let config_window = tauri::WebviewWindowBuilder::new(
            app,
            "config",
            tauri::WebviewUrl::App("pages/config.html".into()),
        )
        .title("Tuya Smart Taskbar - Configuration")
        .inner_size(400.0, 660.0)
        .resizable(false)
        .center()
        .visible(true)
        .build();

        if let Err(e) = config_window {
            tracing::error!("Failed to create config window: {}", e);
        }
    }
}

fn open_about_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("about") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let about_window = tauri::WebviewWindowBuilder::new(
            app,
            "about",
            tauri::WebviewUrl::App("pages/about.html".into()),
        )
        .title("About Tuya Smart Taskbar")
        .inner_size(400.0, 590.0)
        .resizable(false)
        .center()
        .visible(true)
        .build();

        if let Err(e) = about_window {
            tracing::error!("Failed to create about window: {}", e);
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Tuya Smart Taskbar v2.1.0");

    let config_manager = ConfigManager::new();
    let shared_client = create_shared_client();

    if config_manager.is_configured() {
        let cfg = config_manager.get();
        let client_clone = shared_client.clone();

        tauri::async_runtime::block_on(async {
            initialize_client(
                &client_clone,
                cfg.access_key.clone(),
                cfg.secret_key.clone(),
                cfg.base_url.clone(),
            )
            .await;
        });

        tracing::info!("Tuya client initialized");
    }

    let cfg = config_manager.get();
    if let Err(e) = set_auto_launch(cfg.run_on_startup) {
        tracing::warn!("Failed to set auto-launch: {}", e);
    }

    let status_cache: DeviceStatusCache = Arc::new(RwLock::new(HashMap::new()));
    let menu_update_lock: MenuUpdateLock = Arc::new(Mutex::new(()));
    let update_state: SharedUpdateState = create_update_state();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tracing::info!("Second instance detected, focusing existing window");
            open_config_window(app);
        }))
        .manage(shared_client.clone())
        .manage(config_manager)
        .manage(status_cache.clone())
        .manage(update_state.clone())
        .invoke_handler(tauri::generate_handler![
            commands::config::save_config,
            commands::config::get_config,
            commands::config::is_configured,
            commands::config::get_regions,
            commands::devices::fetch_devices,
            commands::devices::fetch_device_status,
            commands::devices::send_device_command,
            commands::devices::toggle_device_state,
            commands::app::get_version,
            commands::app::check_for_update,
            commands::app::open_external,
        ])
        .setup(move |app| {
            let icon = Image::from_bytes(ICON_BYTES).expect("Failed to load tray icon");

            let update_state_for_menu = update_state.clone();
            let initial_menu = tauri::async_runtime::block_on(async {
                tray::build_unconfigured_menu(app.handle(), &update_state_for_menu).await
            })
            .expect("Failed to create initial menu");

            let status_cache_for_event = status_cache.clone();
            let menu_lock_for_event = menu_update_lock.clone();
            let update_state_for_event = update_state.clone();
            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .tooltip("Tuya Smart Taskbar")
                .menu(&initial_menu)
                .on_menu_event(move |app, event| {
                    handle_menu_event(
                        app,
                        event,
                        status_cache_for_event.clone(),
                        menu_lock_for_event.clone(),
                        update_state_for_event.clone(),
                    );
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            let cache_for_init = status_cache.clone();
            let lock_for_init = menu_update_lock.clone();
            let update_state_for_init = update_state.clone();
            tauri::async_runtime::spawn(async move {
                update_tray_menu(
                    &app_handle,
                    false,
                    &cache_for_init,
                    &lock_for_init,
                    &update_state_for_init,
                )
                .await;
            });

            let app_handle = app.handle().clone();
            let cache_for_startup = status_cache.clone();
            let lock_for_startup = menu_update_lock.clone();
            let update_state_for_startup = update_state.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3)).await;
                check_and_notify_update(
                    &app_handle,
                    &update_state_for_startup,
                    Some(&cache_for_startup),
                    Some(&lock_for_startup),
                )
                .await;
            });

            RUNNING.store(true, Ordering::Release);
            let app_handle = app.handle().clone();
            let cache_for_loop = status_cache.clone();
            let lock_for_loop = menu_update_lock.clone();
            let update_state_for_loop = update_state.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(10));
                loop {
                    interval.tick().await;
                    if !RUNNING.load(Ordering::Acquire) {
                        break;
                    }

                    let counter = UPDATE_CHECK_COUNTER.fetch_add(1, Ordering::Relaxed);
                    if counter % UPDATE_CHECK_INTERVAL == 0 && counter > 0 {
                        check_and_notify_update(
                            &app_handle,
                            &update_state_for_loop,
                            Some(&cache_for_loop),
                            Some(&lock_for_loop),
                        )
                        .await;
                    }

                    let result = tokio::time::timeout(
                        Duration::from_secs(15),
                        update_tray_menu(
                            &app_handle,
                            true,
                            &cache_for_loop,
                            &lock_for_loop,
                            &update_state_for_loop,
                        ),
                    )
                    .await;

                    if result.is_err() {
                        tracing::warn!("Auto-refresh timed out, will retry next cycle");
                    }
                }
                tracing::info!("Auto-refresh loop terminated");
            });

            tracing::info!("Application setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("Error while building tauri application");

    app.run(|_app_handle, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            if RUNNING.load(Ordering::Acquire) {
                api.prevent_exit();
            }
        }
    });
}
