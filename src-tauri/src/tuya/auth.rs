use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_nonce() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}

pub fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn sha256_hex(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

pub const EMPTY_BODY_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

pub fn build_string_to_sign(
    method: &str,
    path: &str,
    query_params: Option<&[(&str, &str)]>,
    body: Option<&[u8]>,
) -> String {
    let content_hash = match body {
        Some(b) if !b.is_empty() => sha256_hex(b),
        _ => EMPTY_BODY_HASH.to_string(),
    };

    let url = match query_params {
        Some(params) if !params.is_empty() => {
            let mut sorted_params: Vec<_> = params.iter().collect();
            sorted_params.sort_by(|a, b| a.0.cmp(b.0));
            let query_string = sorted_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", path, query_string)
        }
        _ => path.to_string(),
    };

    let headers_str = "";

    format!(
        "{}\n{}\n{}\n{}",
        method.to_uppercase(),
        content_hash,
        headers_str,
        url
    )
}

pub fn sign_token_request(
    client_id: &str,
    secret: &str,
    timestamp: i64,
    nonce: &str,
    string_to_sign: &str,
) -> String {
    let message = format!("{}{}{}{}", client_id, timestamp, nonce, string_to_sign);
    compute_hmac_sha256(secret, &message)
}

pub fn sign_api_request(
    client_id: &str,
    access_token: &str,
    secret: &str,
    timestamp: i64,
    nonce: &str,
    string_to_sign: &str,
) -> String {
    let message = format!(
        "{}{}{}{}{}",
        client_id, access_token, timestamp, nonce, string_to_sign
    );
    compute_hmac_sha256(secret, &message)
}

fn compute_hmac_sha256(secret: &str, message: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode_upper(result.into_bytes())
}

#[derive(Debug, Clone)]
pub struct SignedHeaders {
    pub client_id: String,
    pub sign: String,
    pub t: String,
    pub nonce: String,
    pub sign_method: String,
    pub access_token: Option<String>,
}

impl SignedHeaders {
    pub fn for_token_request(
        client_id: &str,
        secret: &str,
        method: &str,
        path: &str,
        query_params: Option<&[(&str, &str)]>,
    ) -> Self {
        let timestamp = get_timestamp();
        let nonce = generate_nonce();
        let string_to_sign = build_string_to_sign(method, path, query_params, None);
        let sign = sign_token_request(client_id, secret, timestamp, &nonce, &string_to_sign);

        Self {
            client_id: client_id.to_string(),
            sign,
            t: timestamp.to_string(),
            nonce,
            sign_method: "HMAC-SHA256".to_string(),
            access_token: None,
        }
    }

    pub fn for_api_request(
        client_id: &str,
        access_token: &str,
        secret: &str,
        method: &str,
        path: &str,
        query_params: Option<&[(&str, &str)]>,
        body: Option<&[u8]>,
    ) -> Self {
        let timestamp = get_timestamp();
        let nonce = generate_nonce();
        let string_to_sign = build_string_to_sign(method, path, query_params, body);
        let sign = sign_api_request(
            client_id,
            access_token,
            secret,
            timestamp,
            &nonce,
            &string_to_sign,
        );

        Self {
            client_id: client_id.to_string(),
            sign,
            t: timestamp.to_string(),
            nonce,
            sign_method: "HMAC-SHA256".to_string(),
            access_token: Some(access_token.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_body_hash() {
        assert_eq!(sha256_hex(b""), EMPTY_BODY_HASH);
    }

    #[test]
    fn test_string_to_sign_get_request() {
        let result = build_string_to_sign("GET", "/v1.0/token", Some(&[("grant_type", "1")]), None);
        assert!(result.starts_with("GET\n"));
        assert!(result.contains(EMPTY_BODY_HASH));
        assert!(result.ends_with("/v1.0/token?grant_type=1"));
    }

    #[test]
    fn test_string_to_sign_post_request() {
        let body = b"{\"commands\":[{\"code\":\"switch\",\"value\":true}]}";
        let result = build_string_to_sign("POST", "/v1.0/devices/123/commands", None, Some(body));
        assert!(result.starts_with("POST\n"));
        assert!(!result.contains(EMPTY_BODY_HASH));
        assert!(result.ends_with("/v1.0/devices/123/commands"));
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 32);
    }
}
