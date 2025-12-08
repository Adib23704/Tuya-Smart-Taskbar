use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicI64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

use super::auth::SignedHeaders;
use super::types::{TokenResponse, TokenState, TuyaApiResponse};
use crate::error::AppError;

const TOKEN_REQUEST_TIMEOUT_SECS: u64 = 15;
const TOKEN_CONNECT_TIMEOUT_SECS: u64 = 10;
const MAX_CONSECUTIVE_FAILURES: u32 = 5;
const FAILURE_COOLDOWN_SECS: i64 = 60;

pub struct TokenManager {
    client_id: String,
    secret: String,
    base_url: String,
    http_client: reqwest::Client,
    token_state: Arc<RwLock<Option<TokenState>>>,
    consecutive_failures: AtomicU32,
    last_failure_time: AtomicI64,
}

impl TokenManager {
    pub fn new(client_id: String, secret: String, base_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(TOKEN_REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(TOKEN_CONNECT_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client for token manager");

        Self {
            client_id,
            secret,
            base_url,
            http_client,
            token_state: Arc::new(RwLock::new(None)),
            consecutive_failures: AtomicU32::new(0),
            last_failure_time: AtomicI64::new(0),
        }
    }

    fn check_rate_limit(&self) -> Result<(), AppError> {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures >= MAX_CONSECUTIVE_FAILURES {
            let last_failure = self.last_failure_time.load(Ordering::Relaxed);
            let now = chrono::Utc::now().timestamp();
            let cooldown_remaining = FAILURE_COOLDOWN_SECS - (now - last_failure);

            if cooldown_remaining > 0 {
                tracing::warn!(
                    "Token acquisition rate limited: {} consecutive failures, {} seconds until retry allowed",
                    failures,
                    cooldown_remaining
                );
                return Err(AppError::Network(format!(
                    "Too many token acquisition failures. Please wait {} seconds.",
                    cooldown_remaining
                )));
            }
            // Cooldown expired, reset failure count
            self.consecutive_failures.store(0, Ordering::Relaxed);
        }
        Ok(())
    }

    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        self.last_failure_time.store(
            chrono::Utc::now().timestamp(),
            Ordering::Relaxed,
        );
    }

    pub async fn get_access_token(&self) -> Result<String, AppError> {
        // Check if we have a valid cached token first
        {
            let state = self.token_state.read().await;
            if let Some(ref token) = *state {
                if !token.is_expired() {
                    return Ok(token.access_token.clone());
                }
            }
        }

        // Check rate limit before making network requests
        self.check_rate_limit()?;

        let mut state = self.token_state.write().await;

        // Double-check token validity after acquiring write lock
        if let Some(ref token) = *state {
            if !token.is_expired() {
                return Ok(token.access_token.clone());
            }
            // Try to refresh existing token
            match self.refresh_token_internal(&token.refresh_token).await {
                Ok(new_state) => {
                    self.record_success();
                    let access_token = new_state.access_token.clone();
                    *state = Some(new_state);
                    return Ok(access_token);
                }
                Err(e) => {
                    self.record_failure();
                    tracing::warn!("Token refresh failed: {}, acquiring new token", e);
                }
            }
        }

        // Acquire new token
        match self.acquire_token().await {
            Ok(new_state) => {
                self.record_success();
                let access_token = new_state.access_token.clone();
                *state = Some(new_state);
                Ok(access_token)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
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
}
