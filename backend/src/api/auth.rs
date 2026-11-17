use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::models::{User, Session};
use crate::auth_utils::{
    hash_password, verify_password,
    create_access_token, create_refresh_token,
    decode_token, get_token_username, generate_token_hash,
    create_user_session, revoke_user_session
};
use shared::{RegisterRequest, LoginRequest, TokenResponse, UserResponse, AuthResponse, ChangePasswordRequest, ForgotPasswordRequest, ResetPasswordRequest, PaginatedResponse, SessionResponse};
use chrono::{Utc, Duration};

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct UpdateMeRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    let existing_user = sqlx::query!("SELECT id FROM users WHERE username = $1 OR email = $2", req.username, req.email)
        .fetch_optional(&pool).await?;
    if existing_user.is_some() {
        return Err(AppError::Conflict("Username or email already registered".to_string()));
    }

    let password_hash = hash_password(&req.password)?;
    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO users (username, email, password_hash, role, is_active)
           VALUES ($1, $2, $3, 'user', true)
           RETURNING id, username, email, password_hash, role, is_active, created_at, updated_at"#,
        req.username, req.email, password_hash
    ).fetch_one(&pool).await?;

    let default_quota = 10 * 1024 * 1024 * 1024; // 10GB
    sqlx::query!("INSERT INTO user_storage (user_id, quota_bytes, used_bytes) VALUES ($1, $2, 0)", user.id, default_quota)
        .execute(&pool).await?;

    let access_token = create_access_token(&user.username)?;
    let refresh_token = create_refresh_token(&user.username)?;

    let user_role = user.role();
    let user_id = user.id;
    let username = user.username.clone();
    let email = user.email.clone();
    let is_active = user.is_active;
    let created_at = user.created_at;

    Ok(Json(AuthResponse {
        tokens: TokenResponse {
            access_token,
            refresh_token,
            token_type: "bearer".to_string(),
            expires_in: 900,
        },
        user: UserResponse {
            id: user_id,
            username,
            email,
            role: user_role,
            is_active,
            created_at,
        },
    }))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, role, is_active, created_at, updated_at FROM users WHERE email = $1",
        req.email
    ).fetch_optional(&pool).await?
    .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    if !user.is_active {
        return Err(AppError::Forbidden("Account is inactive".to_string()));
    }

    let device_name = None; // Axum doesn't have easy access to headers here
    let _session = create_user_session(&pool, user.id, device_name, None, None, req.remember_me.unwrap_or(false)).await?;

    let access_token = create_access_token(&user.username)?;
    let refresh_token = create_refresh_token(&user.username)?;

    let user_role = user.role();
    let user_id = user.id;
    let username = user.username.clone();
    let email = user.email.clone();
    let is_active = user.is_active;
    let created_at = user.created_at;

    Ok(Json(AuthResponse {
        tokens: TokenResponse {
            access_token,
            refresh_token,
            token_type: "bearer".to_string(),
            expires_in: 900,
        },
        user: UserResponse {
            id: user_id,
            username,
            email,
            role: user_role,
            is_active,
            created_at,
        },
    }))
}

pub async fn refresh(
    State(pool): State<PgPool>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<AuthResponse>> {
    let claims = decode_token(&req.refresh_token)?;
    if claims.token_type != "refresh" {
        return Err(AppError::Auth("Invalid token type".to_string()));
    }

    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, role, is_active, created_at, updated_at FROM users WHERE username = $1 AND is_active = true",
        claims.sub
    ).fetch_optional(&pool).await?
    .ok_or_else(|| AppError::Auth("User not found or inactive".to_string()))?;

    let access_token = create_access_token(&user.username)?;
    let refresh_token = create_refresh_token(&user.username)?;

    let user_role = user.role();
    let user_id = user.id;
    let username = user.username.clone();
    let email = user.email.clone();
    let is_active = user.is_active;
    let created_at = user.created_at;

    Ok(Json(AuthResponse {
        tokens: TokenResponse {
            access_token,
            refresh_token,
            token_type: "bearer".to_string(),
            expires_in: 900,
        },
        user: UserResponse {
            id: user_id,
            username,
            email,
            role: user_role,
            is_active,
            created_at,
        },
    }))
}

pub async fn logout(
    State(pool): State<PgPool>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    // The token is already validated by middleware, just return success
    Ok(Json(serde_json::json!({ "message": "Successfully logged out" })))
}

pub async fn me(
    AuthUser(user): AuthUser,
) -> AppResult<Json<UserResponse>> {
    Ok(Json(UserResponse {
        id: user.id,
        username: user.username.clone(),
        email: user.email.clone(),
        role: user.role(),
        is_active: user.is_active,
        created_at: user.created_at,
    }))
}

pub async fn update_me(
    State(pool): State<PgPool>,
    AuthUser(user): AuthUser,
    Json(req): Json<UpdateMeRequest>,
) -> AppResult<Json<UserResponse>> {
    let mut updated_user = user.clone();

    if let Some(username) = req.username {
        let exists = sqlx::query!("SELECT 1 FROM users WHERE username = $1 AND id != $2", &username, user.id)
            .fetch_optional(&pool).await?;
        if exists.is_some() {
            return Err(AppError::Conflict("Username already taken".to_string()));
        }
        updated_user.username = username;
    }

    if let Some(email) = req.email {
        let exists = sqlx::query!("SELECT 1 FROM users WHERE email = $1 AND id != $2", &email, user.id)
            .fetch_optional(&pool).await?;
        if exists.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }
        updated_user.email = email;
    }

    let updated = sqlx::query_as!(
        User,
        "UPDATE users SET username = $1, email = $2, updated_at = NOW() WHERE id = $3 RETURNING *",
        updated_user.username, updated_user.email, user.id
    ).fetch_one(&pool).await?;

    Ok(Json(UserResponse {
        id: updated.id,
        username: updated.username,
        email: updated.email,
        role: updated.role(),
        is_active: updated.is_active,
        created_at: updated.created_at,
    }))
}

pub async fn change_password(
    State(pool): State<PgPool>,
    AuthUser(user): AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let db_user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user.id)
        .fetch_one(&pool).await?;

    if !verify_password(&req.current_password, &db_user.password_hash)? {
        return Err(AppError::Validation("Current password is incorrect".to_string()));
    }

    let new_hash = hash_password(&req.new_password)?;
    sqlx::query!("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2", new_hash, user.id)
        .execute(&pool).await?;

    // Revoke all sessions
    sqlx::query!("UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL", user.id)
        .execute(&pool).await?;

    Ok(Json(serde_json::json!({ "message": "Password changed successfully" })))
}

pub async fn forgot_password(
    State(_pool): State<PgPool>,
    Json(_req): Json<ForgotPasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // For security, always return same message
    Ok(Json(serde_json::json!({ "message": "If the email exists, a reset link has been sent" })))
}

pub async fn reset_password(
    State(_pool): State<PgPool>,
    Json(_req): Json<ResetPasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    Err(AppError::Validation("Password reset via email not yet implemented".to_string()))
}

pub async fn list_sessions(
    State(pool): State<PgPool>,
    AuthUser(user): AuthUser,
    axum::extract::Query(params): axum::extract::Query<ListSessionsQuery>,
) -> AppResult<Json<PaginatedResponse<shared::SessionResponse>>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND revoked_at IS NULL",
        user.id
    ).fetch_one(&pool).await?.unwrap_or(0);

    let sessions = sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE user_id = $1 AND revoked_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        user.id, page_size as i64, ((page - 1) * page_size) as i64
    ).fetch_all(&pool).await?;

    let items: Vec<shared::SessionResponse> = sessions.into_iter().map(|s| shared::SessionResponse {
        id: s.id,
        device_name: s.device_name,
        created_at: s.created_at,
        expires_at: s.expires_at,
        revoked_at: s.revoked_at,
        is_current: false,
    }).collect();

    Ok(Json(PaginatedResponse {
        items,
        total,
        page,
        page_size,
        total_pages: (total + page_size - 1) / page_size,
    }))
}

pub async fn revoke_session(
    State(pool): State<PgPool>,
    AuthUser(user): AuthUser,
    axum::extract::Path(session_id): axum::extract::Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    revoke_user_session(&pool, user.id, session_id, None).await?;
    Ok(Json(serde_json::json!({ "message": "Session revoked successfully" })))
}