//! Shared types and DTOs used by both backend and frontend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

impl Default for UserRole {
    fn default() -> Self { Self::User }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

pub type FileListResponse = PaginatedResponse<FileEntry>;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12, max = 128))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    #[serde(flatten)]
    pub tokens: TokenResponse,
    pub user: UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1))]
    pub current_password: String,
    #[validate(length(min = 12, max = 128))]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 12, max = 128))]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFolderRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateFileRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UploadInitiateRequest {
    #[validate(length(min = 1, max = 255))]
    pub filename: String,
    #[validate(range(min = 1))]
    pub size_bytes: i64,
    pub mime_type: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadInitiateResponse {
    pub upload_id: Uuid,
    pub chunk_size_bytes: i64,
    pub total_chunks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadStatusResponse {
    pub upload_id: Uuid,
    pub received_chunks: Vec<i64>,
    pub total_chunks: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShareRequest {
    pub file_id: i64,
    #[validate(email)]
    pub recipient_email: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub permissions: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareResponse {
    pub id: i64,
    pub file_id: i64,
    pub owner_id: i64,
    pub recipient_email: Option<String>,
    pub permissions: String,
    pub share_token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub database: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuota {
    pub used_bytes: i64,
    pub quota_bytes: i64,
    pub available_bytes: i64,
}

impl FileEntry {
    pub fn is_folder(&self) -> bool {
        self.mime_type == "application/vnd.ar3mis.folder" || self.is_folder
    }
}
