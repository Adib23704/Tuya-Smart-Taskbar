use std::sync::Arc;
use tokio::sync::RwLock;

use super::auth::SignedHeaders;
use super::token::TokenManager;
use super::types::{
    TuyaApiResponse, TuyaCommand, TuyaCommandPayload, TuyaDevice, TuyaDeviceStatus, TuyaValue,
};
use crate::error::AppError;

pub struct TuyaClient {
    token_manager: TokenManager,
    http_client: reqwest::Client,
    base_url: String,
    client_id: String,
    secret: String,
}

impl TuyaClient {
    pub fn new(client_id: String, secret: String, base_url: String) -> Self {
        Self {
            token_manager: TokenManager::new(client_id.clone(), secret.clone(), base_url.clone()),
            http_client: reqwest::Client::new(),
            base_url,
            client_id,
            secret,
        }
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        self.request::<T>("GET", path, None, None).await
    }

    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let body_bytes = serde_json::to_vec(body)?;
        self.request("POST", path, None, Some(body_bytes)).await
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        query_params: Option<&[(&str, &str)]>,
        body: Option<Vec<u8>>,
    ) -> Result<T, AppError> {
        let access_token = self.token_manager.get_access_token().await?;

        let headers = SignedHeaders::for_api_request(
            &self.client_id,
            &access_token,
            &self.secret,
            method,
            path,
            query_params,
            body.as_deref(),
        );

        let url = match query_params {
            Some(params) if !params.is_empty() => {
                let mut sorted: Vec<_> = params.iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(b.0));
                let qs = sorted
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                format!("{}{}?{}", self.base_url, path, qs)
            }
            _ => format!("{}{}", self.base_url, path),
        };

        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => return Err(AppError::Network(format!("Invalid method: {}", method))),
        };

        request_builder = request_builder
            .header("client_id", &headers.client_id)
            .header("access_token", headers.access_token.as_deref().unwrap_or(""))
            .header("sign", &headers.sign)
            .header("sign_method", &headers.sign_method)
            .header("t", &headers.t)
            .header("nonce", &headers.nonce);

        if let Some(body_bytes) = body {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .body(body_bytes);
        }

        tracing::debug!("Making {} request to {}", method, url);

        let response = request_builder.send().await?;
        let status = response.status();
        let body_text = response.text().await?;

        tracing::debug!("Response status: {}, body: {}", status, body_text);

        let api_response: TuyaApiResponse<T> = serde_json::from_str(&body_text)
            .map_err(|e| AppError::Parse(format!("{}: {}", e, body_text)))?;

        if !api_response.success {
            if api_response.code == Some(1010) {
                tracing::warn!("Token invalid, invalidating and will retry next request");
                self.token_manager.invalidate().await;
            }
            return Err(AppError::Api {
                code: api_response.code.unwrap_or(-1),
                message: api_response
                    .msg
                    .unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        api_response.result.ok_or(AppError::Api {
            code: -1,
            message: "No result in response".to_string(),
        })
    }

    pub async fn fetch_devices(&self, user_id: &str) -> Result<Vec<TuyaDevice>, AppError> {
        let path = format!("/v1.0/users/{}/devices", user_id);
        self.get(&path).await
    }

    pub async fn fetch_device_status(
        &self,
        device_id: &str,
    ) -> Result<Vec<TuyaDeviceStatus>, AppError> {
        let path = format!("/v1.0/devices/{}/status", device_id);
        self.get(&path).await
    }

    pub async fn send_device_commands(
        &self,
        device_id: &str,
        commands: Vec<TuyaCommand>,
    ) -> Result<bool, AppError> {
        let path = format!("/v1.0/devices/{}/commands", device_id);
        let payload = TuyaCommandPayload { commands };
        self.post(&path, &payload).await
    }

    pub async fn send_device_command(
        &self,
        device_id: &str,
        code: &str,
        value: TuyaValue,
    ) -> Result<bool, AppError> {
        self.send_device_commands(
            device_id,
            vec![TuyaCommand {
                code: code.to_string(),
                value,
            }],
        )
        .await
    }

    pub async fn toggle_device_state(
        &self,
        device_id: &str,
        code: &str,
        current_value: bool,
    ) -> Result<bool, AppError> {
        self.send_device_command(device_id, code, TuyaValue::Boolean(!current_value))
            .await
    }
}

pub type SharedTuyaClient = Arc<RwLock<Option<TuyaClient>>>;

pub fn create_shared_client() -> SharedTuyaClient {
    Arc::new(RwLock::new(None))
}

pub async fn initialize_client(
    shared: &SharedTuyaClient,
    client_id: String,
    secret: String,
    base_url: String,
) {
    let client = TuyaClient::new(client_id, secret, base_url);
    let mut guard = shared.write().await;
    *guard = Some(client);
}
