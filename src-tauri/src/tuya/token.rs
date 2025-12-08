use std::sync::Arc;
use tokio::sync::RwLock;

use super::auth::SignedHeaders;
use super::types::{TokenResponse, TokenState, TuyaApiResponse};
use crate::error::AppError;

pub struct TokenManager {
    client_id: String,
    secret: String,
    base_url: String,
    http_client: reqwest::Client,
    token_state: Arc<RwLock<Option<TokenState>>>,
}

impl TokenManager {
    pub fn new(client_id: String, secret: String, base_url: String) -> Self {
        Self {
            client_id,
            secret,
            base_url,
            http_client: reqwest::Client::new(),
            token_state: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_access_token(&self) -> Result<String, AppError> {
        {
            let state = self.token_state.read().await;
            if let Some(ref token) = *state {
                if !token.is_expired() {
                    return Ok(token.access_token.clone());
                }
            }
        }

        let mut state = self.token_state.write().await;

        if let Some(ref token) = *state {
            if !token.is_expired() {
                return Ok(token.access_token.clone());
            }
            match self.refresh_token_internal(&token.refresh_token).await {
                Ok(new_state) => {
                    let access_token = new_state.access_token.clone();
                    *state = Some(new_state);
                    return Ok(access_token);
                }
                Err(e) => {
                    tracing::warn!("Token refresh failed: {}, acquiring new token", e);
                }
            }
        }

        let new_state = self.acquire_token().await?;
        let access_token = new_state.access_token.clone();
        *state = Some(new_state);
        Ok(access_token)
    }

    async fn acquire_token(&self) -> Result<TokenState, AppError> {
        let path = "/v1.0/token";
        let query_params = [("grant_type", "1")];

        let headers =
            SignedHeaders::for_token_request(&self.client_id, &self.secret, "GET", path, Some(&query_params));

        let url = format!("{}{}?grant_type=1", self.base_url, path);

        tracing::debug!("Acquiring token from {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("client_id", &headers.client_id)
            .header("sign", &headers.sign)
            .header("sign_method", &headers.sign_method)
            .header("t", &headers.t)
            .header("nonce", &headers.nonce)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        tracing::debug!("Token response status: {}, body: {}", status, body);

        let api_response: TuyaApiResponse<TokenResponse> =
            serde_json::from_str(&body).map_err(|e| AppError::Parse(format!("{}: {}", e, body)))?;

        if !api_response.success {
            return Err(AppError::Api {
                code: api_response.code.unwrap_or(-1),
                message: api_response
                    .msg
                    .unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        let token = api_response.result.ok_or(AppError::Api {
            code: -1,
            message: "No token in response".to_string(),
        })?;

        let now = chrono::Utc::now().timestamp();
        Ok(TokenState {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at: now + token.expire_time,
        })
    }

    async fn refresh_token_internal(&self, refresh_token: &str) -> Result<TokenState, AppError> {
        let path = format!("/v1.0/token/{}", refresh_token);

        let headers =
            SignedHeaders::for_token_request(&self.client_id, &self.secret, "GET", &path, None);

        let url = format!("{}{}", self.base_url, path);

        tracing::debug!("Refreshing token at {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("client_id", &headers.client_id)
            .header("sign", &headers.sign)
            .header("sign_method", &headers.sign_method)
            .header("t", &headers.t)
            .header("nonce", &headers.nonce)
            .send()
            .await?;

        let api_response: TuyaApiResponse<TokenResponse> = response.json().await.map_err(|e| {
            AppError::Parse(e.to_string())
        })?;

        if !api_response.success {
            return Err(AppError::Api {
                code: api_response.code.unwrap_or(-1),
                message: api_response
                    .msg
                    .unwrap_or_else(|| "Token refresh failed".to_string()),
            });
        }

        let token = api_response.result.ok_or(AppError::Api {
            code: -1,
            message: "No token in refresh response".to_string(),
        })?;

        let now = chrono::Utc::now().timestamp();
        Ok(TokenState {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at: now + token.expire_time,
        })
    }

    pub async fn invalidate(&self) {
        let mut state = self.token_state.write().await;
        *state = None;
    }

    pub async fn has_valid_token(&self) -> bool {
        let state = self.token_state.read().await;
        if let Some(ref token) = *state {
            !token.is_expired()
        } else {
            false
        }
    }
}
