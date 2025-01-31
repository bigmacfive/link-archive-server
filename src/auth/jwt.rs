use std::time::{SystemTime, UNIX_EPOCH};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;
use std::collections::HashSet;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static BLACKLISTED_TOKENS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub device_info: Option<DeviceInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub user_agent: String,
    pub ip_address: String,
    pub device_id: Option<String>,
}

#[derive(Debug)]
pub struct TokenMetadata {
    pub user_id: Uuid,
    pub expires_at: usize,
    pub issued_at: usize,
    pub token_id: String,
    pub device_info: Option<DeviceInfo>,
}

pub fn create_token(user_id: Uuid, device_info: Option<DeviceInfo>) -> Result<String, AppError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::InternalServerError)?
        .as_secs() as usize;

    let expiration = now + 24 * 3600; // 24 hours from now
    let token_id = Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user_id,
        exp: expiration,
        iat: now,
        jti: token_id,
        device_info,
    };

    let secret = std::env::var("JWT_SECRET").map_err(|_| AppError::InternalServerError)?;
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::AuthError(e.to_string()))
}

pub fn verify_token(token: &str) -> Result<TokenMetadata, AppError> {
    // Check if token is blacklisted
    if BLACKLISTED_TOKENS.lock().unwrap().contains(token) {
        return Err(AppError::AuthError("Token has been revoked".to_string()));
    }

    let secret = std::env::var("JWT_SECRET").map_err(|_| AppError::InternalServerError)?;
    
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.validate_nbf = true;
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => 
            AppError::AuthError("Token has expired".to_string()),
        _ => AppError::AuthError(e.to_string())
    })?;

    Ok(TokenMetadata {
        user_id: token_data.claims.sub,
        expires_at: token_data.claims.exp,
        issued_at: token_data.claims.iat,
        token_id: token_data.claims.jti,
        device_info: token_data.claims.device_info,
    })
}

pub fn revoke_token(token: &str) -> Result<(), AppError> {
    let mut blacklist = BLACKLISTED_TOKENS.lock().unwrap();
    blacklist.insert(token.to_string());
    Ok(())
}

pub fn is_token_blacklisted(token: &str) -> bool {
    BLACKLISTED_TOKENS.lock().unwrap().contains(token)
}

pub fn cleanup_blacklist() {
    let mut blacklist = BLACKLISTED_TOKENS.lock().unwrap();
    blacklist.clear();
}