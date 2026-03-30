use tauri::AppHandle;

use crate::error::CommandResult;
use crate::update::{self, UpdateInfo};

#[tauri::command]
pub fn get_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> CommandResult<UpdateInfo> {
    update::check_for_update(&app)
        .await
        .ok_or_else(|| crate::error::SerializableError {
            error_type: "network".to_string(),
            message: "Failed to check for updates".to_string(),
            code: None,
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
