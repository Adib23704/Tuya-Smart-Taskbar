use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::AppHandle;
use tokio::sync::RwLock;

const UPDATE_CHECK_URL: &str = "https://raw.githubusercontent.com/Adib23704/Tuya-Smart-Taskbar/refs/heads/master/package.json";
const DOWNLOAD_URL: &str = "https://github.com/Adib23704/Tuya-Smart-Taskbar/releases/latest";
const CHECK_INTERVAL: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
}

#[derive(Debug, Default)]
pub struct UpdateState {
    pub update_available: bool,
    pub latest_version: Option<String>,
    pub last_check: Option<Instant>,
    pub notification_shown: bool,
}

pub type SharedUpdateState = Arc<RwLock<UpdateState>>;

pub fn create_update_state() -> SharedUpdateState {
    Arc::new(RwLock::new(UpdateState::default()))
}

pub async fn check_for_update(app: &AppHandle) -> Option<UpdateInfo> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to create HTTP client: {}", e);
            return None;
        }
    };

    let response = match client.get(UPDATE_CHECK_URL).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to fetch update info: {}", e);
            return None;
        }
    };

    let package: serde_json::Value = match response.json().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to parse update response: {}", e);
            return None;
        }
    };

    let latest_version = match package["version"].as_str() {
        Some(v) => v.to_string(),
        None => {
            tracing::error!("No version field in package.json");
            return None;
        }
    };

    let current_version = app.package_info().version.to_string();
    let available = is_newer_version(&latest_version, &current_version);

    tracing::info!(
        "Update check: current={}, latest={}, available={}",
        current_version,
        latest_version,
        available
    );

    Some(UpdateInfo {
        available,
        current_version,
        latest_version,
        download_url: DOWNLOAD_URL.to_string(),
    })
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let latest_parts = parse_version(latest);
    let current_parts = parse_version(current);

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

pub async fn should_check_for_update(state: &SharedUpdateState) -> bool {
    let guard = state.read().await;
    match guard.last_check {
        None => true,
        Some(last) => last.elapsed() >= CHECK_INTERVAL,
    }
}

pub async fn update_state(
    state: &SharedUpdateState,
    update_info: &UpdateInfo,
) -> (bool, bool) {
    let mut guard = state.write().await;
    let was_available = guard.update_available;
    let notification_shown = guard.notification_shown;

    guard.update_available = update_info.available;
    guard.latest_version = Some(update_info.latest_version.clone());
    guard.last_check = Some(Instant::now());

    let is_new_detection = update_info.available && !was_available;
    let should_notify = is_new_detection && !notification_shown;

    if should_notify {
        guard.notification_shown = true;
    }

    (is_new_detection, should_notify)
}

pub async fn get_update_info(state: &SharedUpdateState) -> Option<(bool, String)> {
    let guard = state.read().await;
    if guard.update_available {
        guard.latest_version.clone().map(|v| (true, v))
    } else {
        None
    }
}

pub fn get_download_url() -> &'static str {
    DOWNLOAD_URL
}
