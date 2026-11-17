use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::services::storage::{StorageService, UploadSessions};
use shared::{UploadInitiateRequest, UploadInitiateResponse, UploadStatusResponse, FileEntry};

pub async fn initiate_upload(
    State((pool, upload_sessions)): State<(PgPool, UploadSessions)>,
    AuthUser(user): AuthUser,
    Json(req): Json<UploadInitiateRequest>,
) -> AppResult<Json<UploadInitiateResponse>> {
    let storage = StorageService::new(pool, upload_sessions);
    let (upload_id, total_chunks) = storage.initiate_upload(
        user.id,
        req.filename,
        req.size_bytes,
        req.mime_type,
        req.parent_id,
    ).await?;

    Ok(Json(UploadInitiateResponse {
        upload_id,
        chunk_size_bytes: 1024 * 1024,
        total_chunks,
    }))
}

pub async fn upload_chunk(
    State((_pool, _upload_sessions)): State<(PgPool, UploadSessions)>,
    AuthUser(_user): AuthUser,
    Path(_upload_id): Path<Uuid>,
    _body: axum::body::Bytes,
) -> AppResult<Json<serde_json::Value>> {
    // This is the old route - kept for compatibility but redirect to upload_chunk_with_query
    Ok(Json(serde_json::json!({ "message": "Use upload_chunk_with_query with chunk_number query param" })))
}

#[derive(serde::Deserialize)]
pub struct UploadChunkQuery {
    pub chunk_number: i32,
}

pub async fn upload_chunk_with_query(
    State((pool, upload_sessions)): State<(PgPool, UploadSessions)>,
    AuthUser(_user): AuthUser,
    Path(upload_id): Path<Uuid>,
    axum::extract::Query(query): axum::extract::Query<UploadChunkQuery>,
    body: axum::body::Bytes,
) -> AppResult<Json<serde_json::Value>> {
    let storage = StorageService::new(pool, upload_sessions);
    storage.upload_chunk(upload_id, query.chunk_number, body.to_vec()).await?;
    Ok(Json(serde_json::json!({ "message": format!("Chunk {} uploaded successfully", query.chunk_number) })))
}

pub async fn get_upload_status(
    State((pool, upload_sessions)): State<(PgPool, UploadSessions)>,
    AuthUser(_user): AuthUser,
    Path(upload_id): Path<Uuid>,
) -> AppResult<Json<UploadStatusResponse>> {
    let storage = StorageService::new(pool, upload_sessions);
    let session = storage.get_upload_status(upload_id).await?
        .ok_or_else(|| AppError::UploadNotFound)?;

    Ok(Json(UploadStatusResponse {
        upload_id,
        received_chunks: session.received_chunks,
        total_chunks: session.total_chunks as i64,
        status: session.status,
    }))
}

pub async fn complete_upload(
    State((pool, upload_sessions)): State<(PgPool, UploadSessions)>,
    AuthUser(_user): AuthUser,
    Path(upload_id): Path<Uuid>,
) -> AppResult<Json<FileEntry>> {
    let storage = StorageService::new(pool, upload_sessions);
    let file = storage.complete_upload(upload_id).await?;

    Ok(Json(FileEntry {
        id: file.id,
        owner_id: file.owner_id,
        parent_id: file.parent_id,
        name: file.name,
        storage_key: file.storage_key,
        mime_type: file.mime_type,
        size_bytes: file.size_bytes,
        checksum: file.checksum,
        version: file.version,
        is_folder: file.is_folder,
        deleted_at: file.deleted_at,
        created_at: file.created_at,
        updated_at: file.updated_at,
    }))
}