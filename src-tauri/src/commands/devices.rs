use tauri::State;

use crate::config::ConfigManager;
use crate::error::{AppError, CommandResult, SerializableError};
use crate::tuya::{SharedTuyaClient, TuyaDevice, TuyaDeviceStatus, TuyaValue};

#[tauri::command]
pub async fn fetch_devices(
    client: State<'_, SharedTuyaClient>,
    config: State<'_, ConfigManager>,
) -> CommandResult<Vec<TuyaDevice>> {
    let guard = client.read().await;
    let tuya_client = guard
        .as_ref()
        .ok_or_else(|| SerializableError::from(AppError::NotConfigured))?;

    let user_id = config.get_user_id().ok_or_else(|| {
        SerializableError::from(AppError::Config("User ID not configured".to_string()))
    })?;

    tuya_client
        .fetch_devices(&user_id)
        .await
        .map_err(SerializableError::from)
}

#[tauri::command]
pub async fn fetch_device_status(
    device_id: String,
    client: State<'_, SharedTuyaClient>,
) -> CommandResult<Vec<TuyaDeviceStatus>> {
    let guard = client.read().await;
    let tuya_client = guard
        .as_ref()
        .ok_or_else(|| SerializableError::from(AppError::NotConfigured))?;

    tuya_client
        .fetch_device_status(&device_id)
        .await
        .map_err(SerializableError::from)
}

#[tauri::command]
pub async fn send_device_command(
    device_id: String,
    code: String,
    value: serde_json::Value,
    client: State<'_, SharedTuyaClient>,
) -> CommandResult<bool> {
    let guard = client.read().await;
    let tuya_client = guard
        .as_ref()
        .ok_or_else(|| SerializableError::from(AppError::NotConfigured))?;

    let tuya_value = match value {
        serde_json::Value::Bool(b) => TuyaValue::Boolean(b),
        serde_json::Value::String(s) => TuyaValue::String(s),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                TuyaValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                TuyaValue::Float(f)
            } else {
                return Err(SerializableError {
                    error_type: "parse".to_string(),
                    message: "Invalid number value".to_string(),
                    code: None,
                });
            }
        }
        _ => {
            return Err(SerializableError {
                error_type: "parse".to_string(),
                message: "Unsupported value type".to_string(),
                code: None,
            });
        }
    };

    tuya_client
        .send_device_command(&device_id, &code, tuya_value)
        .await
        .map_err(SerializableError::from)
}

#[tauri::command]
pub async fn toggle_device_state(
    device_id: String,
    code: String,
    current_value: bool,
    client: State<'_, SharedTuyaClient>,
) -> CommandResult<bool> {
    let guard = client.read().await;
    let tuya_client = guard
        .as_ref()
        .ok_or_else(|| SerializableError::from(AppError::NotConfigured))?;

    tuya_client
        .toggle_device_state(&device_id, &code, current_value)
        .await
        .map_err(SerializableError::from)
}
