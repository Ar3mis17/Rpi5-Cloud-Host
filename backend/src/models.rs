//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use shared::UserRole;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn role(&self) -> UserRole {
        match self.role.as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct File {
    pub id: i64,
    pub owner_id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub storage_key: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum: String,
    pub version: i32,
    pub is_folder: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub device_name: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Share {
    pub id: i64,
    pub file_id: i64,
    pub owner_id: i64,
    pub recipient_email: Option<String>,
    pub permissions: String,
    pub share_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserStorage {
    pub user_id: i64,
    pub quota_bytes: i64,
    pub used_bytes: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: i64,
    pub user_id: i64,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UploadSession {
    pub upload_id: Uuid,
    pub user_id: i64,
    pub filename: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub parent_id: Option<i64>,
    pub total_chunks: i32,
    pub received_chunks: Vec<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
