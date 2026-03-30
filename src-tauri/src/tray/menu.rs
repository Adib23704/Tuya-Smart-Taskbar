use std::collections::HashMap;
use std::sync::Arc;

use futures::future::join_all;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Wry,
};
use tokio::sync::RwLock;

use crate::config::ConfigManager;
use crate::error::AppError;
use crate::tuya::{
    SharedTuyaClient, TuyaDevice, TuyaDeviceStatus, TuyaValue, AC_FAN_SPEED_LEVELS, AC_MODES,
    FAN_SPEED_LEVELS, TEMP_MAX, TEMP_MIN,
};
use crate::update::SharedUpdateState;

pub type MenuItemRegistry = Arc<RwLock<HashMap<String, CheckMenuItem<Wry>>>>;

pub fn create_menu_registry() -> MenuItemRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}

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
    registry: &mut HashMap<String, CheckMenuItem<Wry>>,
) -> Result<Submenu<Wry>, AppError> {
    let submenu =
        Submenu::new(app, &device.name, true).map_err(|e| AppError::Tray(e.to_string()))?;

    for s in status {
        match s.code.as_str() {
            _ if s.value.as_bool().is_some() => {
                let checked = s.value.as_bool().unwrap_or(false);
                let label = format_label(&s.code);
                let id = format!("toggle:{}:{}", device.id, s.code);

                let item = CheckMenuItem::with_id(app, &id, &label, true, checked, None::<&str>)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                let registry_key = format!("{}:{}", device.id, s.code);
                registry.insert(registry_key, item.clone());
                submenu
                    .append(&item)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }

            "fan_speed_percent" => {
                let current = s.value.as_i64().unwrap_or(1) as i32;
                let speed_submenu = Submenu::new(app, "Fan Speed", true)
                    .map_err(|e| AppError::Tray(e.to_string()))?;

                for level in 1..=FAN_SPEED_LEVELS {
                    let id = format!("set:{}:fan_speed_percent:{}", device.id, level);
                    let item = CheckMenuItem::with_id(
                        app,
                        &id,
                        level.to_string(),
                        true,
                        current == level,
                        None::<&str>,
                    )
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                    let registry_key = format!("{}:fan_speed_percent:{}", device.id, level);
                    registry.insert(registry_key, item.clone());
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
                    let id = format!("set:{}:temp_set:{}", device.id, temp);
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
                    let registry_key = format!("{}:temp_set:{}", device.id, temp);
                    registry.insert(registry_key, item.clone());
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
                    let id = format!("set:{}:windspeed:{}", device.id, level);
                    let item = CheckMenuItem::with_id(
                        app,
                        &id,
                        level.to_string(),
                        true,
                        current == level,
                        None::<&str>,
                    )
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                    let registry_key = format!("{}:windspeed:{}", device.id, level);
                    registry.insert(registry_key, item.clone());
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
                    let id = format!("set:{}:mode:{}", device.id, mode);
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
                    let registry_key = format!("{}:mode:{}", device.id, mode);
                    registry.insert(registry_key, item.clone());
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
        let no_controls = MenuItem::with_id(app, "no_controls", "No controls", false, None::<&str>)
            .map_err(|e| AppError::Tray(e.to_string()))?;
        submenu
            .append(&no_controls)
            .map_err(|e| AppError::Tray(e.to_string()))?;
    }

    Ok(submenu)
}

async fn append_update_item(
    app: &AppHandle,
    menu: &Menu<Wry>,
    update_state: &SharedUpdateState,
) -> Result<bool, AppError> {
    let guard = update_state.read().await;
    if guard.update_available {
        if let Some(ref version) = guard.latest_version {
            let update_item = MenuItem::with_id(
                app,
                "open_update",
                format!("Update Available (v{})", version),
                true,
                None::<&str>,
            )
            .map_err(|e| AppError::Tray(e.to_string()))?;
            menu.append(&update_item)
                .map_err(|e| AppError::Tray(e.to_string()))?;
            menu.append(
                &PredefinedMenuItem::separator(app).map_err(|e| AppError::Tray(e.to_string()))?,
            )
            .map_err(|e| AppError::Tray(e.to_string()))?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub async fn build_unconfigured_menu(
    app: &AppHandle,
    update_state: &SharedUpdateState,
) -> Result<Menu<Wry>, AppError> {
    let menu = Menu::new(app).map_err(|e| AppError::Tray(e.to_string()))?;

    append_update_item(app, &menu, update_state).await?;

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

pub async fn build_error_menu(
    app: &AppHandle,
    update_state: &SharedUpdateState,
) -> Result<Menu<Wry>, AppError> {
    let menu = Menu::new(app).map_err(|e| AppError::Tray(e.to_string()))?;

    append_update_item(app, &menu, update_state).await?;

    let error_item = MenuItem::with_id(app, "error", "Error loading devices", false, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh Devices", true, None::<&str>)
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
    menu.append(&refresh_item)
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
    update_state: &SharedUpdateState,
) -> Result<
    (
        Menu<Wry>,
        HashMap<String, Vec<TuyaDeviceStatus>>,
        HashMap<String, CheckMenuItem<Wry>>,
    ),
    AppError,
> {
    let menu = Menu::new(app).map_err(|e| AppError::Tray(e.to_string()))?;
    let mut device_statuses: HashMap<String, Vec<TuyaDeviceStatus>> = HashMap::new();
    let mut registry: HashMap<String, CheckMenuItem<Wry>> = HashMap::new();

    append_update_item(app, &menu, update_state).await?;

    let user_id = config
        .get_user_id()
        .ok_or(AppError::Config("User ID not configured".to_string()))?;

    let guard = client.read().await;
    let tuya_client = guard.as_ref().ok_or(AppError::NotConfigured)?;

    let devices = tuya_client.fetch_devices(&user_id).await?;
    let online_devices: Vec<_> = devices.iter().filter(|d| d.online).collect();

    let status_futures: Vec<_> = online_devices
        .iter()
        .map(|d| tuya_client.fetch_device_status(&d.id))
        .collect();

    let statuses: Vec<Result<Vec<TuyaDeviceStatus>, AppError>> = join_all(status_futures).await;

    for (device, status_result) in online_devices.iter().zip(statuses.into_iter()) {
        match status_result {
            Ok(status) => {
                device_statuses.insert(device.id.clone(), status.clone());
                let submenu = build_device_submenu(app, device, &status, &mut registry)?;
                menu.append(&submenu)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
            }
            Err(e) => {
                tracing::warn!("Failed to fetch status for device {}: {}", device.id, e);
                let submenu = Submenu::new(app, format!("{} (error)", device.name), true)
                    .map_err(|e| AppError::Tray(e.to_string()))?;
                let error_item = MenuItem::with_id(
                    app,
                    format!("error_{}", device.id),
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

    if online_devices.is_empty() {
        let offline_count = devices.len();
        let label = if offline_count == 0 {
            "No devices found".to_string()
        } else {
            format!("All {} device(s) offline", offline_count)
        };
        let no_devices = MenuItem::with_id(app, "no_devices", &label, false, None::<&str>)
            .map_err(|e| AppError::Tray(e.to_string()))?;
        menu.append(&no_devices)
            .map_err(|e| AppError::Tray(e.to_string()))?;
    }

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| AppError::Tray(e.to_string()))?)
        .map_err(|e| AppError::Tray(e.to_string()))?;

    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh Devices", true, None::<&str>)
        .map_err(|e| AppError::Tray(e.to_string()))?;
    menu.append(&refresh_item)
        .map_err(|e| AppError::Tray(e.to_string()))?;

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

    Ok((menu, device_statuses, registry))
}

pub fn parse_command_id(id: &str) -> Option<(String, String, String)> {
    if let Some(rest) = id.strip_prefix("toggle:") {
        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string(), String::new()));
        }
        return None;
    }

    if let Some(rest) = id.strip_prefix("set:") {
        let parts: Vec<&str> = rest.splitn(3, ':').collect();
        if parts.len() == 3 {
            return Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ));
        }
        return None;
    }

    // Legacy cmd: prefix
    if let Some(rest) = id.strip_prefix("cmd:") {
        let parts: Vec<&str> = rest.splitn(3, ':').collect();
        if parts.len() == 3 {
            return Some((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ));
        }
        return None;
    }

    None
}

/// Returns true if the device set or the status codes changed between old and new caches.
/// Value-only changes return false (those can be updated in-place).
pub fn is_structural_change(
    old: &HashMap<String, Vec<TuyaDeviceStatus>>,
    new: &HashMap<String, Vec<TuyaDeviceStatus>>,
) -> bool {
    if old.len() != new.len() {
        return true;
    }

    for (device_id, old_statuses) in old {
        match new.get(device_id) {
            None => return true,
            Some(new_statuses) => {
                if old_statuses.len() != new_statuses.len() {
                    return true;
                }
                for (old_s, new_s) in old_statuses.iter().zip(new_statuses.iter()) {
                    if old_s.code != new_s.code {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Updates check menu items in-place by comparing old and new status caches.
/// Returns the number of items that were updated.
pub fn update_menu_items_in_place(
    registry: &HashMap<String, CheckMenuItem<Wry>>,
    old_statuses: &HashMap<String, Vec<TuyaDeviceStatus>>,
    new_statuses: &HashMap<String, Vec<TuyaDeviceStatus>>,
) -> usize {
    let mut updated = 0;

    for (device_id, new_status_list) in new_statuses {
        let old_status_list = match old_statuses.get(device_id) {
            Some(v) => v,
            None => continue,
        };

        for (new_s, old_s) in new_status_list.iter().zip(old_status_list.iter()) {
            if new_s.value == old_s.value {
                continue;
            }

            match new_s.code.as_str() {
                _ if new_s.value.as_bool().is_some() => {
                    let key = format!("{}:{}", device_id, new_s.code);
                    if let Some(item) = registry.get(&key) {
                        let checked = new_s.value.as_bool().unwrap_or(false);
                        item.set_checked(checked).ok();
                        updated += 1;
                    }
                }

                "fan_speed_percent" => {
                    let old_val = old_s.value.as_i64().unwrap_or(1) as i32;
                    let new_val = new_s.value.as_i64().unwrap_or(1) as i32;
                    for level in 1..=FAN_SPEED_LEVELS {
                        let key = format!("{}:fan_speed_percent:{}", device_id, level);
                        if let Some(item) = registry.get(&key) {
                            let was_checked = old_val == level;
                            let is_checked = new_val == level;
                            if was_checked != is_checked {
                                item.set_checked(is_checked).ok();
                                updated += 1;
                            }
                        }
                    }
                }

                "temp_set" => {
                    let old_val = old_s.value.as_i64().unwrap_or(20) as i32;
                    let new_val = new_s.value.as_i64().unwrap_or(20) as i32;
                    for temp in TEMP_MIN..=TEMP_MAX {
                        let key = format!("{}:temp_set:{}", device_id, temp);
                        if let Some(item) = registry.get(&key) {
                            let was_checked = old_val == temp;
                            let is_checked = new_val == temp;
                            if was_checked != is_checked {
                                item.set_checked(is_checked).ok();
                                updated += 1;
                            }
                        }
                    }
                }

                "windspeed" => {
                    let old_val = old_s.value.as_i64().unwrap_or(1) as i32;
                    let new_val = new_s.value.as_i64().unwrap_or(1) as i32;
                    for level in 1..=AC_FAN_SPEED_LEVELS {
                        let key = format!("{}:windspeed:{}", device_id, level);
                        if let Some(item) = registry.get(&key) {
                            let was_checked = old_val == level;
                            let is_checked = new_val == level;
                            if was_checked != is_checked {
                                item.set_checked(is_checked).ok();
                                updated += 1;
                            }
                        }
                    }
                }

                "mode" => {
                    let old_val = old_s.value.as_string().unwrap_or("");
                    let new_val = new_s.value.as_string().unwrap_or("");
                    for mode in AC_MODES {
                        let key = format!("{}:mode:{}", device_id, mode);
                        if let Some(item) = registry.get(&key) {
                            let was_checked = old_val == *mode;
                            let is_checked = new_val == *mode;
                            if was_checked != is_checked {
                                item.set_checked(is_checked).ok();
                                updated += 1;
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }

    updated
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
