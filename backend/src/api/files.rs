use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::models::{File, UserStorage};
use crate::services::storage::StorageService;
use shared::{CreateFolderRequest, FileEntry, PaginatedResponse, UpdateFileRequest};

#[derive(Deserialize)]
pub struct ListFilesQuery {
    parent_id: Option<i64>,
    page: Option<i64>,
    page_size: Option<i64>,
}

fn file_to_entry(f: File) -> FileEntry {
    FileEntry {
        id: f.id,
        owner_id: f.owner_id,
        parent_id: f.parent_id,
        name: f.name,
        storage_key: f.storage_key,
        mime_type: f.mime_type,
        size_bytes: f.size_bytes,
        checksum: f.checksum,
        version: f.version,
        is_folder: f.mime_type == "application/vnd.ar3mis.folder",
        deleted_at: f.deleted_at,
        created_at: f.created_at,
        updated_at: f.updated_at,
    }
}

pub async fn list_files(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Query(params): Query<ListFilesQuery>,
) -> AppResult<Json<PaginatedResponse<FileEntry>>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(50).clamp(1, 100);
    let parent_id = params.parent_id;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM files WHERE owner_id = $1 AND deleted_at IS NULL AND ($2 IS NULL OR parent_id = $2)",
        user.id, parent_id
    ).fetch_one(&pool).await?.unwrap_or(0);

    let files = sqlx::query_as!(
        File,
        "SELECT * FROM files WHERE owner_id = $1 AND deleted_at IS NULL AND ($2 IS NULL OR parent_id = $2) ORDER BY is_folder DESC, name ASC LIMIT $3 OFFSET $4",
        user.id, parent_id, page_size as i64, ((page - 1) * page_size) as i64
    ).fetch_all(&pool).await?;

    Ok(Json(PaginatedResponse {
        items: files.into_iter().map(file_to_entry).collect(),
        total,
        page,
        page_size,
        total_pages: (total + page_size - 1) / page_size,
    }))
}

pub async fn create_folder(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateFolderRequest>,
) -> AppResult<Json<FileEntry>> {
    let storage_key = format!("{}/{}", uuid::Uuid::new_v4(), req.name);
    let file = sqlx::query_as!(
        File,
        r#"INSERT INTO files (owner_id, parent_id, name, storage_key, mime_type, size_bytes, checksum, is_folder)
           VALUES ($1, $2, $3, $4, 'application/vnd.ar3mis.folder', 0, '', true)
           RETURNING *"#,
        user.id, req.parent_id, req.name, storage_key
    ).fetch_one(&pool).await?;

    let storage = StorageService::new()?;
    let path = storage.user_file_path(user.id, &file.storage_key);
    std::fs::create_dir_all(&path)?;

    Ok(Json(file_to_entry(file)))
}

pub async fn get_file(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<FileEntry>> {
    let file = sqlx::query_as!(File, "SELECT * FROM files WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL", id, user.id)
        .fetch_optional(&pool).await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;
    Ok(Json(file_to_entry(file)))
}

pub async fn update_file(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateFileRequest>,
) -> AppResult<Json<FileEntry>> {
    let file = sqlx::query_as!(File, "SELECT * FROM files WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL", id, user.id)
        .fetch_optional(&pool).await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let name = req.name.unwrap_or(file.name);
    let updated = sqlx::query_as!(
        File,
        "UPDATE files SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        name, id
    ).fetch_one(&pool).await?;

    Ok(Json(file_to_entry(updated)))
}

pub async fn delete_file(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query!("UPDATE files SET deleted_at = NOW() WHERE id = $1 AND owner_id = $2", id, user.id)
        .execute(&pool).await?;
    Ok(Json(serde_json::json!({ "message": "File deleted" })))
}

pub async fn download_file(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<axum::response::Response> {
    let file = sqlx::query_as!(File, "SELECT * FROM files WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL", id, user.id)
        .fetch_optional(&pool).await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let storage = StorageService::new()?;
    let path = storage.user_file_path(user.id, &file.storage_key);

    if !path.exists() {
        return Err(AppError::NotFound("File not found on disk".to_string()));
    }

    let file_data = std::fs::read(&path)?;
    let mime = file.mime_type.clone();
    let filename = file.name.clone();

    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .body(axum::body::Body::from(file_data))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(response)
}

pub async fn get_storage_quota(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<shared::StorageQuota>> {
    let storage = sqlx::query_as!(UserStorage, "SELECT * FROM user_storage WHERE user_id = $1", user.id)
        .fetch_optional(&pool).await?
        .ok_or_else(|| AppError::NotFound("Storage quota not found".to_string()))?;

    Ok(Json(shared::StorageQuota {
        used_bytes: storage.used_bytes,
        quota_bytes: storage.quota_bytes,
        available_bytes: storage.quota_bytes - storage.used_bytes,
    }))
}
