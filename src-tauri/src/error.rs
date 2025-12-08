use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Failed to parse response: {0}")]
    Parse(String),

    #[error("API error (code: {code}): {message}")]
    Api { code: i32, message: String },

    #[error("Client not configured")]
    NotConfigured,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tray error: {0}")]
    Tray(String),
}

#[derive(Debug, Serialize, Clone)]
pub struct SerializableError {
    pub error_type: String,
    pub message: String,
    pub code: Option<i32>,
}

impl From<AppError> for SerializableError {
    fn from(err: AppError) -> Self {
        match &err {
            AppError::Api { code, message } => SerializableError {
                error_type: "api".to_string(),
                message: message.clone(),
                code: Some(*code),
            },
            _ => SerializableError {
                error_type: format!("{:?}", err)
                    .split('(')
                    .next()
                    .unwrap_or("unknown")
                    .to_lowercase(),
                message: err.to_string(),
                code: None,
            },
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

pub type CommandResult<T> = Result<T, SerializableError>;

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
