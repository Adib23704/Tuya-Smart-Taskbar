use tauri::State;

use crate::config::{get_available_regions, set_auto_launch, AppConfig, ConfigManager, RegionInfo};
use crate::error::{CommandResult, SerializableError};
use crate::tuya::{initialize_client, SharedTuyaClient};

#[tauri::command]
pub async fn save_config(
    new_config: AppConfig,
    client: State<'_, SharedTuyaClient>,
    config_manager: State<'_, ConfigManager>,
) -> CommandResult<()> {
    config_manager
        .save(&new_config)
        .map_err(|e| SerializableError::from(e))?;

    if let Err(e) = set_auto_launch(new_config.run_on_startup) {
        tracing::warn!("Failed to set auto-launch: {}", e);
    }

    if new_config.is_configured() {
        initialize_client(
            &client,
            new_config.access_key.clone(),
            new_config.secret_key.clone(),
            new_config.base_url.clone(),
        )
        .await;
        tracing::info!("Tuya client reinitialized with new config");
    } else {
        let mut guard = client.write().await;
        *guard = None;
        tracing::info!("Tuya client cleared (config incomplete)");
    }

    Ok(())
}

#[tauri::command]
pub fn get_config(config_manager: State<'_, ConfigManager>) -> CommandResult<AppConfig> {
    Ok(config_manager.get())
}

#[tauri::command]
pub fn is_configured(config_manager: State<'_, ConfigManager>) -> bool {
    config_manager.is_configured()
}

#[tauri::command]
pub fn get_regions() -> Vec<RegionInfo> {
    get_available_regions()
}
