//! Storage service

use crate::errors::AppError;
use crate::models::{File, UploadSession};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;
use sha2::{Sha256, Digest};

pub type UploadSessions = Arc<Mutex<HashMap<Uuid, UploadSession>>>;

pub struct StorageService {
    pool: PgPool,
    upload_sessions: UploadSessions,
}

impl StorageService {
    pub fn new(pool: PgPool, upload_sessions: UploadSessions) -> Self {
        Self { pool, upload_sessions }
    }

    pub async fn check_quota(&self, user_id: i64, additional_bytes: i64) -> Result<(), AppError> {
        let quota: Option<i64> = sqlx::query_scalar!(
            "SELECT quota_bytes FROM user_storage WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        let used: i64 = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM files WHERE owner_id = $1 AND deleted_at IS NULL",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        let limit = quota.unwrap_or(10 * 1024 * 1024 * 1024); // 10GB default
        if used + additional_bytes > limit {
            return Err(AppError::QuotaExceeded);
        }
        Ok(())
    }

    pub async fn initiate_upload(
        &self,
        user_id: i64,
        filename: String,
        size_bytes: i64,
        mime_type: Option<String>,
        parent_id: Option<i64>,
    ) -> Result<(Uuid, i64), AppError> {
        self.check_quota(user_id, size_bytes).await?;

        let upload_id = Uuid::new_v4();
        let chunk_size = 1024 * 1024; // 1MB
        let total_chunks = (size_bytes + chunk_size - 1) / chunk_size;

        let session = UploadSession {
            upload_id,
            user_id,
            filename,
            size_bytes,
            mime_type,
            parent_id,
            chunks_received: 0,
            total_chunks,
            file_hash: Sha256::new(),
            created_at: chrono::Utc::now(),
        };

        self.upload_sessions.lock().await.insert(upload_id, session);
        Ok((upload_id, total_chunks))
    }

    pub async fn upload_chunk(
        &self,
        upload_id: Uuid,
        chunk_number: i64,
        data: Vec<u8>,
    ) -> Result<(), AppError> {
        let mut sessions = self.upload_sessions.lock().await;
        let session = sessions.get_mut(&upload_id).ok_or(AppError::UploadNotFound)?;

        if chunk_number != session.chunks_received + 1 {
            return Err(AppError::InvalidChunkOrder);
        }

        session.file_hash.update(&data);
        session.chunks_received = chunk_number;
        Ok(())
    }

    pub async fn complete_upload(&self, upload_id: Uuid) -> Result<File, AppError> {
        let mut sessions = self.upload_sessions.lock().await;
        let session = sessions.remove(&upload_id).ok_or(AppError::UploadNotFound)?;

        if session.chunks_received != session.total_chunks {
            return Err(AppError::UploadIncomplete);
        }

        let file_hash = format!("{:x}", session.file_hash.finalize_reset());

        let file = sqlx::query_as!(
            File,
            r#"INSERT INTO files (owner_id, parent_id, name, mime_type, size_bytes, sha256_hash, storage_path, is_folder)
               VALUES ($1, $2, $3, $4, $5, $6, $7, false)
               RETURNING id, owner_id, parent_id, name, mime_type, size_bytes, sha256_hash, storage_path, is_folder, deleted_at, created_at, updated_at"#,
            session.user_id,
            session.parent_id,
            session.filename,
            session.mime_type,
            session.size_bytes,
            file_hash,
            format!("uploads/{}/{}", session.user_id, session.filename)
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(file)
    }

    pub async fn get_upload_status(&self, upload_id: Uuid) -> Result<Option<UploadSession>, AppError> {
        let sessions = self.upload_sessions.lock().await;
        Ok(sessions.get(&upload_id).cloned())
    }
}
