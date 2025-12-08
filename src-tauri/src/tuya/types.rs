use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuyaDevice {
    pub id: String,
    pub name: String,
    pub online: bool,
    pub category: String,
    pub product_id: String,
    pub product_name: String,
    pub local_key: String,
    pub sub: bool,
    pub uuid: String,
    pub owner_id: String,
    #[serde(default)]
    pub ip: String,
    pub time_zone: String,
    pub create_time: i64,
    pub update_time: i64,
    pub active_time: i64,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuyaDeviceStatus {
    pub code: String,
    pub value: TuyaValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TuyaValue {
    Boolean(bool),
    String(String),
    Integer(i64),
    Float(f64),
}

impl TuyaValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            TuyaValue::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            TuyaValue::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            TuyaValue::Integer(v) => Some(*v),
            TuyaValue::Float(v) => Some(*v as i64),
            _ => None,
        }
    }
}

impl std::fmt::Display for TuyaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuyaValue::Boolean(v) => write!(f, "{}", v),
            TuyaValue::String(v) => write!(f, "{}", v),
            TuyaValue::Integer(v) => write!(f, "{}", v),
            TuyaValue::Float(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuyaCommand {
    pub code: String,
    pub value: TuyaValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuyaCommandPayload {
    pub commands: Vec<TuyaCommand>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TuyaApiResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    pub code: Option<i32>,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expire_time: i64,
}

#[derive(Debug, Clone)]
pub struct TokenState {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

impl TokenState {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= (self.expires_at - 300)
    }
}

pub const AC_MODES: &[&str] = &["auto", "cold", "dry", "wind"];

pub const FAN_SPEED_LEVELS: i32 = 5;

pub const AC_FAN_SPEED_LEVELS: i32 = 4;

pub const TEMP_MIN: i32 = 16;
pub const TEMP_MAX: i32 = 30;
