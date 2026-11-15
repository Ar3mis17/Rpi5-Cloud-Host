//! Authentication middleware

use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use crate::auth::{decode_token, get_token_username, generate_token_hash};
use crate::errors::{AppError, AppResult};
use crate::models::User;

#[derive(Clone)]
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<User>() {
            return Ok(AuthUser(user.clone()));
        }
        Err(AppError::Auth("Not authenticated".to_string()))
    }
}

pub async fn auth_middleware(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => return Err(AppError::Auth("Missing authorization header".to_string())),
    };

    let username = get_token_username(token)?;
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
           FROM users WHERE username = $1"#, username
    ).fetch_optional(&pool).await?.ok_or_else(|| AppError::Auth("User not found".to_string()))?;

    if !user.is_active {
        return Err(AppError::Forbidden("Account is inactive".to_string()));
    }

    let session_valid = sqlx::query!(
        "SELECT 1 FROM sessions WHERE user_id = $1 AND token_hash = $2 AND revoked_at IS NULL AND expires_at > NOW()",
        user.id, generate_token_hash(token)
    ).fetch_optional(&pool).await?.is_some();

    if !session_valid {
        return Err(AppError::Auth("Session expired or revoked".to_string()));
    }

    request.extensions_mut().insert(user.clone());
    Ok(next.run(request).await)
}

pub async fn optional_auth_middleware(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        if let Ok(username) = get_token_username(token) {
            if let Ok(Some(user)) = sqlx::query_as!(
                User,
                r#"SELECT id, username, email, password_hash, role, is_active, created_at, updated_at
                   FROM users WHERE username = $1 AND is_active = true"#, username
            ).fetch_optional(&pool).await {
                let session_valid = sqlx::query!(
                    "SELECT 1 FROM sessions WHERE user_id = $1 AND token_hash = $2 AND revoked_at IS NULL AND expires_at > NOW()",
                    user.id, generate_token_hash(token)
                ).fetch_optional(&pool).await.unwrap_or(None).is_some();

                if session_valid {
                    request.extensions_mut().insert(user);
                }
            }
        }
    }
    next.run(request).await
}
