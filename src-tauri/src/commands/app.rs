use serde::Serialize;
use tauri::AppHandle;

use crate::error::CommandResult;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
}

#[tauri::command]
pub fn get_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> CommandResult<UpdateInfo> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://raw.githubusercontent.com/Adib23704/Tuya-Smart-Taskbar/refs/heads/master/package.json")
        .send()
        .await
        .map_err(|e| crate::error::SerializableError {
            error_type: "network".to_string(),
            message: e.to_string(),
            code: None,
        })?;

    let package: serde_json::Value = response
        .json()
        .await
        .map_err(|e| crate::error::SerializableError {
            error_type: "parse".to_string(),
            message: e.to_string(),
            code: None,
        })?;

    let latest_version = package["version"]
        .as_str()
        .unwrap_or("0.0.0")
        .to_string();
    let current_version = app.package_info().version.to_string();

    Ok(UpdateInfo {
        available: latest_version != current_version,
        current_version,
        latest_version,
        download_url: "https://github.com/Adib23704/Tuya-Smart-Taskbar/releases/latest".to_string(),
    })
}

#[tauri::command]
pub fn open_external(url: String) -> CommandResult<()> {
    let parsed = url::Url::parse(&url).map_err(|_| crate::error::SerializableError {
        error_type: "validation".to_string(),
        message: "Invalid URL".to_string(),
        code: None,
    })?;

    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return Err(crate::error::SerializableError {
            error_type: "validation".to_string(),
            message: "Only HTTP(S) URLs allowed".to_string(),
            code: None,
        });
    }

    open::that(&url).map_err(|e| crate::error::SerializableError {
        error_type: "io".to_string(),
        message: e.to_string(),
        code: None,
    })
}
