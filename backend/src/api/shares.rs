use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::models::{File, Share};
use shared::{CreateShareRequest, ShareResponse};

#[derive(Deserialize)]
pub struct AccessSharedFileQuery {
    token: String,
}

pub async fn create_share(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateShareRequest>,
) -> AppResult<Json<ShareResponse>> {
    let file = sqlx::query_as!(File, "SELECT * FROM files WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL", req.file_id, user.id)
        .fetch_optional(&pool).await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let share_token = uuid::Uuid::new_v4().to_string();
    let expires_at = req.expires_at;

    let share = sqlx::query_as!(
        Share,
        r#"INSERT INTO shares (file_id, owner_id, recipient_email, permissions, share_token, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING *"#,
        file.id, user.id, req.recipient_email, req.permissions.unwrap_or_else(|| "read".to_string()), share_token, expires_at
    ).fetch_one(&pool).await?;

    Ok(Json(ShareResponse {
        id: share.id,
        file_id: share.file_id,
        owner_id: share.owner_id,
        recipient_email: share.recipient_email,
        permissions: share.permissions,
        share_token: share.share_token,
        expires_at: share.expires_at,
        created_at: share.created_at,
    }))
}

pub async fn list_shares(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<Vec<ShareResponse>>> {
    let shares = sqlx::query_as!(Share, "SELECT * FROM shares WHERE owner_id = $1 ORDER BY created_at DESC", user.id)
        .fetch_all(&pool).await?;

    Ok(Json(shares.into_iter().map(|s| ShareResponse {
        id: s.id,
        file_id: s.file_id,
        owner_id: s.owner_id,
        recipient_email: s.recipient_email,
        permissions: s.permissions,
        share_token: s.share_token,
        expires_at: s.expires_at,
        created_at: s.created_at,
    }).collect()))
}

pub async fn revoke_share(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Path(share_id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query!("DELETE FROM shares WHERE id = $1 AND owner_id = $2", share_id, user.id)
        .execute(&pool).await?;
    Ok(Json(serde_json::json!({ "message": "Share revoked" })))
}

pub async fn access_shared_file(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    Query(params): Query<AccessSharedFileQuery>,
    Path(token): Path<String>,
) -> AppResult<Json<shared::FileEntry>> {
    let share = sqlx::query_as!(Share, "SELECT * FROM shares WHERE share_token = $1", token)
        .fetch_optional(&pool).await?
        .ok_or_else(|| AppError::NotFound("Share not found".to_string()))?;

    if let Some(expires) = share.expires_at {
        if expires < chrono::Utc::now() {
            return Err(AppError::NotFound("Share has expired".to_string()));
        }
    }

    let file = sqlx::query_as!(File, "SELECT * FROM files WHERE id = $1 AND deleted_at IS NULL", share.file_id)
        .fetch_optional(&pool).await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    Ok(Json(shared::FileEntry {
        id: file.id,
        owner_id: file.owner_id,
        parent_id: file.parent_id,
        name: file.name,
        storage_key: file.storage_key,
        mime_type: file.mime_type,
        size_bytes: file.size_bytes,
        checksum: file.checksum,
        version: file.version,
        is_folder: file.mime_type == "application/vnd.ar3mis.folder",
        deleted_at: file.deleted_at,
        created_at: file.created_at,
        updated_at: file.updated_at,
    }))
}
