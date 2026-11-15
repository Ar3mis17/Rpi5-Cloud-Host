//! Authentication utilities

use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::get_config;
use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
}

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub fn create_access_token(username: &str) -> AppResult<String> {
    let config = get_config();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
    let exp = now + config.access_token_expire_secs() as usize;
    let claims = Claims { sub: username.to_string(), exp, iat: now, token_type: "access".to_string() };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(config.jwt_secret()))?;
    Ok(token)
}

pub fn create_refresh_token(username: &str) -> AppResult<String> {
    let config = get_config();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
    let exp = now + config.refresh_token_expire_secs() as usize;
    let claims = Claims { sub: username.to_string(), exp, iat: now, token_type: "refresh".to_string() };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(config.jwt_secret()))?;
    Ok(token)
}

pub fn decode_token(token: &str) -> AppResult<Claims> {
    let config = get_config();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(config.jwt_secret()), &validation)?;
    Ok(token_data.claims)
}

pub fn get_token_username(token: &str) -> AppResult<String> {
    let claims = decode_token(token)?;
    Ok(claims.sub)
}

pub fn is_token_valid(token: &str) -> bool {
    decode_token(token).is_ok()
}

pub fn generate_token_hash(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn create_user_session(
    pool: &sqlx::PgPool,
    user_id: i64,
    device_name: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    remember_me: bool,
) -> AppResult<crate::models::Session> {
    let token = uuid::Uuid::new_v4().to_string();
    let token_hash = generate_token_hash(&token);
    let expires_delta = chrono::Duration::days(if remember_me { 30 } else { 7 });
    let session = sqlx::query_as!(
        crate::models::Session,
        r#"INSERT INTO sessions (user_id, token_hash, device_name, ip_address, user_agent, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, user_id, token_hash, device_name, ip_address, user_agent, expires_at, revoked_at, created_at"#,
        user_id, token_hash, device_name, ip_address, user_agent, chrono::Utc::now() + expires_delta
    ).fetch_one(pool).await?;
    Ok(session)
}

pub async fn revoke_user_session(
    pool: &sqlx::PgPool,
    user_id: i64,
    session_id: i64,
    current_session_id: Option<i64>,
) -> AppResult<bool> {
    if let Some(current) = current_session_id {
        if current == session_id {
            return Err(AppError::Validation("Cannot revoke current session".to_string()));
        }
    }
    sqlx::query!("UPDATE sessions SET revoked_at = NOW() WHERE id = $1 AND user_id = $2", session_id, user_id)
        .execute(pool).await?;
    Ok(true)
}
