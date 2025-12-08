use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub access_key: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub user_id: String,
    #[serde(default = "default_true")]
    pub run_on_startup: bool,
}

fn default_true() -> bool {
    true
}

impl AppConfig {
    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty()
            && !self.access_key.is_empty()
            && !self.secret_key.is_empty()
            && !self.user_id.is_empty()
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: RwLock<AppConfig>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let config = Self::load_from_path(&config_path).unwrap_or_default();

        tracing::info!("Config path: {:?}", config_path);
        tracing::debug!("Config loaded, is_configured: {}", config.is_configured());

        Self {
            config_path,
            config: RwLock::new(config),
        }
    }

    fn get_config_path() -> PathBuf {
        if let Some(local_data) = directories::BaseDirs::new() {
            let config_dir = local_data.data_local_dir().join("Tuya Smart Taskbar");
            if let Err(e) = fs::create_dir_all(&config_dir) {
                tracing::warn!("Failed to create config directory: {}", e);
            }
            config_dir.join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    fn load_from_path(path: &PathBuf) -> Option<AppConfig> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn get(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    pub fn get_user_id(&self) -> Option<String> {
        let config = self.config.read().unwrap();
        if config.user_id.is_empty() {
            None
        } else {
            Some(config.user_id.clone())
        }
    }

    pub fn save(&self, new_config: &AppConfig) -> Result<(), AppError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        {
            let mut config = self.config.write().unwrap();
            *config = new_config.clone();
        }

        let content = serde_json::to_string_pretty(new_config)?;
        fs::write(&self.config_path, content)?;

        tracing::info!("Configuration saved to {:?}", self.config_path);
        Ok(())
    }

    pub fn is_configured(&self) -> bool {
        self.config.read().unwrap().is_configured()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub url: &'static str,
}

pub fn get_available_regions() -> Vec<RegionInfo> {
    vec![
        RegionInfo {
            id: "central_europe",
            name: "Central Europe",
            url: "https://openapi.tuyaeu.com",
        },
        RegionInfo {
            id: "western_europe",
            name: "Western Europe",
            url: "https://openapi-weaz.tuyaeu.com",
        },
        RegionInfo {
            id: "china",
            name: "China",
            url: "https://openapi.tuyacn.com",
        },
        RegionInfo {
            id: "western_america",
            name: "Western America",
            url: "https://openapi.tuyaus.com",
        },
        RegionInfo {
            id: "eastern_america",
            name: "Eastern America",
            url: "https://openapi-ueaz.tuyaus.com",
        },
        RegionInfo {
            id: "india",
            name: "India",
            url: "https://openapi.tuyain.com",
        },
        RegionInfo {
            id: "singapore",
            name: "Singapore",
            url: "https://openapi-sg.iotbing.com",
        },
    ]
}

pub fn set_auto_launch(enabled: bool) -> Result<(), AppError> {
    let exe_path = std::env::current_exe().map_err(|e| AppError::Config(e.to_string()))?;
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let auto_launch =
        auto_launch::AutoLaunch::new("Tuya Smart Taskbar", &exe_path_str, &[] as &[&str]);

    if enabled {
        auto_launch
            .enable()
            .map_err(|e| AppError::Config(e.to_string()))?;
        tracing::info!("Auto-launch enabled");
    } else {
        if auto_launch.is_enabled().unwrap_or(false) {
            auto_launch
                .disable()
                .map_err(|e| AppError::Config(e.to_string()))?;
            tracing::info!("Auto-launch disabled");
        }
    }

    Ok(())
}
