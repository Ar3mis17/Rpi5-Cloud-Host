use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use shared::{UserResponse, FileEntry, PaginatedResponse};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadProgress {
    pub upload_id: Uuid,
    pub filename: String,
    pub size_bytes: i64,
    pub uploaded_bytes: i64,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct AuthStore {
    pub user: RwSignal<Option<UserResponse>>,
    pub is_authenticated: RwSignal<bool>,
    pub is_loading: RwSignal<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct FileStore {
    pub files: RwSignal<Vec<FileEntry>>,
    pub current_folder: RwSignal<Option<i64>>,
    pub breadcrumbs: RwSignal<Vec<Breadcrumb>>,
    pub pagination: RwSignal<Pagination>,
    pub is_loading: RwSignal<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct UploadStore {
    pub uploads: RwSignal<HashMap<Uuid, UploadProgress>>,
}

impl AuthStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FileStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl UploadStore {
    pub fn new() -> Self {
        Self::default()
    }
}

thread_local! {
    pub static AUTH_STORE: AuthStore = AuthStore::new();
    pub static FILE_STORE: FileStore = FileStore::new();
    pub static UPLOAD_STORE: UploadStore = UploadStore::new();
}
