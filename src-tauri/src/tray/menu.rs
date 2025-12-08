use std::collections::HashMap;

use futures::future::join_all;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Wry,
};

use crate::config::ConfigManager;
use crate::error::AppError;
use crate::tuya::{
    SharedTuyaClient, TuyaDevice, TuyaDeviceStatus, TuyaValue, AC_FAN_SPEED_LEVELS,
    AC_MODES, FAN_SPEED_LEVELS, TEMP_MAX, TEMP_MIN,
};

fn format_label(code: &str) -> String {
    code.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn build_device_submenu(
    app: &AppHandle,
    device: &TuyaDevice,
    status: &[TuyaDeviceStatus],
) -> Result<Submenu<Wry>, AppError> {
    let submenu = Submenu::new(app, &device.name, true)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    for s in status {
        match s.code.as_str() {
            _ if s.value.as_bool().is_some() => {
                let checked = s.value.as_bool().unwrap_or(false);
                let label = format_label(&s.code);
                let id = format!("cmd:{}:{}:{}", device.id, s.code, !checked);

                let item = CheckMenuItem::with_id(app, &id, &label, true, checked, None::<&str>)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                submenu
                    .append(&item)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }

            "fan_speed_percent" => {
                let current = s.value.as_i64().unwrap_or(1) as i32;
                let speed_submenu = Submenu::new(app, "Fan Speed", true)
                    .map_err(|e| AppError::Tray(e.to_string()))?;

                for level in 1..=FAN_SPEED_LEVELS {
                    let id = format!("cmd:{}:fan_speed_percent:{}", device.id, level);
                    let item = CheckMenuItem::with_id(
                        app,
                        &id,
                        &level.to_string(),
                        true,
                        current == level,
                        None::<&str>,
                    )
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                    speed_submenu
                        .append(&item)
                        .map_err(|e| AppError::Tray(e.to_string()))?;
                }
                submenu
                    .append(&speed_submenu)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }

            "temp_set" => {
                let current = s.value.as_i64().unwrap_or(20) as i32;
                let temp_submenu = Submenu::new(app, "Temperature", true)
                    .map_err(|e| AppError::Tray(e.to_string()))?;

                for temp in TEMP_MIN..=TEMP_MAX {
                    let id = format!("cmd:{}:temp_set:{}", device.id, temp);
                    let label = format!("{}°C", temp);
                    let item = CheckMenuItem::with_id(
                        app,
                        &id,
                        &label,
                        true,
                        current == temp,
                        None::<&str>,
                    )
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                    temp_submenu
                        .append(&item)
                        .map_err(|e| AppError::Tray(e.to_string()))?;
                }
                submenu
                    .append(&temp_submenu)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }

            "windspeed" => {
                let current = s.value.as_i64().unwrap_or(1) as i32;
                let speed_submenu = Submenu::new(app, "AC Fan Speed", true)
                    .map_err(|e| AppError::Tray(e.to_string()))?;

                for level in 1..=AC_FAN_SPEED_LEVELS {
                    let id = format!("cmd:{}:windspeed:{}", device.id, level);
                    let item = CheckMenuItem::with_id(
                        app,
                        &id,
                        &level.to_string(),
                        true,
                        current == level,
                        None::<&str>,
                    )
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                    speed_submenu
                        .append(&item)
                        .map_err(|e| AppError::Tray(e.to_string()))?;
                }
                submenu
                    .append(&speed_submenu)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }

            "mode" => {
                let current = s.value.as_string().unwrap_or("");
                let mode_submenu = Submenu::new(app, "AC Mode", true)
                    .map_err(|e| AppError::Tray(e.to_string()))?;

                for mode in AC_MODES {
                    let id = format!("cmd:{}:mode:{}", device.id, mode);
                    let label = format_label(mode);
                    let item = CheckMenuItem::with_id(
                        app,
                        &id,
                        &label,
                        true,
                        current == *mode,
                        None::<&str>,
                    )
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                    mode_submenu
                        .append(&item)
                        .map_err(|e| AppError::Tray(e.to_string()))?;
                }
                submenu
                    .append(&mode_submenu)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }

            _ => {}
        }
    }

    let items = submenu.items().map_err(|e| AppError::Tray(e.to_string()))?;
    if items.is_empty() {
        let no_controls =
            MenuItem::with_id(app, "no_controls", "No controls", false, None::<&str>)
                .map_err(|e| AppError::Tray(e.to_string()))?;
        submenu
            .append(&no_controls)
            .map_err(|e| AppError::Tray(e.to_string()))?;
    }

    Ok(submenu)
}

pub fn build_unconfigured_menu(app: &AppHandle) -> Result<Menu<Wry>, AppError> {
    let menu = Menu::new(app).map_err(|e| AppError::Tray(e.to_string()))?;

    let config_item =
        MenuItem::with_id(app, "open_config", "Open Configuration", true, None::<&str>)
            .map_err(|e| AppError::Tray(e.to_string()))?;
    let about_item = MenuItem::with_id(app, "open_about", "About", true, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    menu.append(&config_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&about_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| AppError::Tray(e.to_string()))?)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&quit_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    Ok(menu)
}

pub fn build_error_menu(app: &AppHandle) -> Result<Menu<Wry>, AppError> {
    let menu = Menu::new(app).map_err(|e| AppError::Tray(e.to_string()))?;

    let error_item = MenuItem::with_id(app, "error", "Error loading devices", false, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    let config_item =
        MenuItem::with_id(app, "open_config", "Open Configuration", true, None::<&str>)
            .map_err(|e| AppError::Tray(e.to_string()))?;
    let about_item = MenuItem::with_id(app, "open_about", "About", true, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    menu.append(&error_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| AppError::Tray(e.to_string()))?)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&config_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&about_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| AppError::Tray(e.to_string()))?)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&quit_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    Ok(menu)
}

pub async fn build_device_menu_with_cache(
    app: &AppHandle,
    client: &SharedTuyaClient,
    config: &ConfigManager,
) -> Result<(Menu<Wry>, HashMap<String, Vec<TuyaDeviceStatus>>), AppError> {
    let menu = Menu::new(app).map_err(|e| AppError::Tray(e.to_string()))?;
    let mut device_statuses: HashMap<String, Vec<TuyaDeviceStatus>> = HashMap::new();

    let user_id = config
        .get_user_id()
        .ok_or(AppError::Config("User ID not configured".to_string()))?;

    let guard = client.read().await;
    let tuya_client = guard.as_ref().ok_or(AppError::NotConfigured)?;

    let devices = tuya_client.fetch_devices(&user_id).await?;
    let online_devices: Vec<_> = devices.iter().filter(|d| d.online).collect();

    // Fetch all device statuses in parallel for better performance
    let status_futures: Vec<_> = online_devices
        .iter()
        .map(|d| tuya_client.fetch_device_status(&d.id))
        .collect();

    let statuses: Vec<Result<Vec<TuyaDeviceStatus>, AppError>> = join_all(status_futures).await;

    // Build menu items with fetched statuses
    for (device, status_result) in online_devices.iter().zip(statuses.into_iter()) {
        match status_result {
            Ok(status) => {
                device_statuses.insert(device.id.clone(), status.clone());
                let submenu = build_device_submenu(app, device, &status)?;
                menu.append(&submenu)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }
            Err(e) => {
                tracing::warn!("Failed to fetch status for device {}: {}", device.id, e);
                // Add device with error indicator
                let submenu = Submenu::new(app, format!("{} (error)", device.name), true)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                let error_item = MenuItem::with_id(
                    app,
                    &format!("error_{}", device.id),
                    "Failed to load status",
                    false,
                    None::<&str>,
                )
                .map_err(|e| AppError::Tray(e.to_string()))?;
                submenu
                    .append(&error_item)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                menu.append(&submenu)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }
        }
    }

    if !online_devices.is_empty() {
        menu.append(
            &PredefinedMenuItem::separator(app).map_err(|e| AppError::Tray(e.to_string()))?,
        )
        .map_err(|e| AppError::Tray(e.to_string()))?;
    }

    let config_item =
        MenuItem::with_id(app, "open_config", "Open Configuration", true, None::<&str>)
            .map_err(|e| AppError::Tray(e.to_string()))?;
    let about_item = MenuItem::with_id(app, "open_about", "About", true, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    menu.append(&config_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&about_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| AppError::Tray(e.to_string()))?)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&quit_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    Ok((menu, device_statuses))
}

pub fn parse_command_id(id: &str) -> Option<(String, String, String)> {
    if !id.starts_with("cmd:") {
        return None;
    }

    let parts: Vec<&str> = id[4..].splitn(3, ':').collect();
    if parts.len() == 3 {
        Some((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    } else {
        None
    }
}

pub fn parse_value(value_str: &str) -> TuyaValue {
    if value_str == "true" {
        return TuyaValue::Boolean(true);
    }
    if value_str == "false" {
        return TuyaValue::Boolean(false);
    }

    if let Ok(i) = value_str.parse::<i64>() {
        return TuyaValue::Integer(i);
    }

    TuyaValue::String(value_str.to_string())
}
