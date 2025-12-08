#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{
    image::Image,
    menu::MenuEvent,
    tray::TrayIconBuilder,
    AppHandle, Manager, RunEvent, WindowEvent,
};

mod commands;
mod config;
mod error;
mod tray;
mod tuya;

use config::{set_auto_launch, ConfigManager};
use tuya::{create_shared_client, initialize_client, SharedTuyaClient};

static RUNNING: AtomicBool = AtomicBool::new(false);

// Embed icons at compile time
const ICON_BYTES: &[u8] = include_bytes!("../icons/icon.ico");
const LOADING_ICON_BYTES: &[u8] = include_bytes!("../icons/loading.ico");

async fn update_tray_menu(app: &AppHandle, is_auto_refresh: bool) {
    let config_manager = app.state::<ConfigManager>();
    let client = app.state::<SharedTuyaClient>();

    if !is_auto_refresh {
        if let Some(tray) = app.tray_by_id("main") {
            if let Ok(icon) = Image::from_bytes(LOADING_ICON_BYTES) {
                let _ = tray.set_icon(Some(icon));
            }
        }
    }

    let menu = if !config_manager.is_configured() {
        tray::build_unconfigured_menu(app)
    } else {
        match tray::build_device_menu(app, &client, &config_manager).await {
            Ok(menu) => Ok(menu),
            Err(e) => {
                tracing::error!("Error building device menu: {}", e);
                tray::build_error_menu(app)
            }
        }
    };

    if !is_auto_refresh {
        if let Some(tray) = app.tray_by_id("main") {
            if let Ok(icon) = Image::from_bytes(ICON_BYTES) {
                let _ = tray.set_icon(Some(icon));
            }
        }
    }

    if let Ok(menu) = menu {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    tracing::debug!("Menu event: {}", id);

    match id {
        "open_config" => {
            open_config_window(app);
        }
        "open_about" => {
            open_about_window(app);
        }
        "quit" => {
            RUNNING.store(false, Ordering::Relaxed);
            app.exit(0);
        }
        _ if id.starts_with("cmd:") => {
            if let Some((device_id, code, value_str)) = tray::parse_command_id(id) {
                let value = tray::parse_value(&value_str);
                let app_handle = app.clone();

                tauri::async_runtime::spawn(async move {
                    let client = app_handle.state::<SharedTuyaClient>();
                    let guard = client.read().await;

                    if let Some(tuya_client) = guard.as_ref() {
                        match tuya_client.send_device_command(&device_id, &code, value).await {
                            Ok(_) => {
                                tracing::info!("Command sent: {}:{}", code, value_str);
                                update_tray_menu(&app_handle, false).await;
                            }
                            Err(e) => {
                                tracing::error!("Failed to send command: {}", e);
                            }
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

    tracing::info!("Starting Tuya Smart Taskbar v2.0.0");

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

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tracing::info!("Second instance detected, focusing existing window");
            open_config_window(app);
        }))
        .manage(shared_client.clone())
        .manage(config_manager)
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
        .setup(|app| {
            let icon = Image::from_bytes(ICON_BYTES)
                .expect("Failed to load tray icon");

            let initial_menu = tray::build_unconfigured_menu(app.handle())
                .expect("Failed to create initial menu");

            let _tray = TrayIconBuilder::with_id("main")
                .icon(icon)
                .tooltip("Tuya Smart Taskbar")
                .menu(&initial_menu)
                .on_menu_event(handle_menu_event)
                .build(app)?;

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                update_tray_menu(&app_handle, false).await;
            });

            RUNNING.store(true, Ordering::Relaxed);
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    if !RUNNING.load(Ordering::Relaxed) {
                        break;
                    }
                    update_tray_menu(&app_handle, true).await;
                }
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
            if RUNNING.load(Ordering::Relaxed) {
                api.prevent_exit();
            }
        }
    });
}
