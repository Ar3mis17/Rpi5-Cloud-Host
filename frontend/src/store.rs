use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use shared::{UserResponse, FileEntry, PaginatedResponse};
use web_sys::File as WebSysFile;
use crate::api;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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

    pub fn login(&self, access_token: String, refresh_token: String, user: UserResponse) {
        self.user.set(Some(user));
        self.is_authenticated.set(true);
        self.is_loading.set(false);
        api::set_token(&access_token);
        api::set_refresh_token(&refresh_token);
    }

    pub fn logout(&self) {
        self.user.set(None);
        self.is_authenticated.set(false);
        self.is_loading.set(false);
        api::clear_token();
    }
}

impl FileStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_files(&self, response: PaginatedResponse<FileEntry>) {
        self.files.set(response.items);
        self.pagination.set(Pagination {
            page: response.page,
            page_size: response.page_size,
            total: response.total,
            total_pages: response.total_pages,
        });
        self.is_loading.set(false);
    }

    pub fn add_file(&self, file: FileEntry) {
        self.files.update(|files| files.push(file));
    }

    pub fn set_loading(&self, loading: bool) {
        self.is_loading.set(loading);
    }

    pub fn set_current_folder(&self, folder_id: Option<i64>) {
        self.current_folder.set(folder_id);
    }

    pub fn set_breadcrumbs(&self, breadcrumbs: Vec<Breadcrumb>) {
        self.breadcrumbs.set(breadcrumbs);
    }

    pub fn set_pagination(&self, pagination: Pagination) {
        self.pagination.set(pagination);
    }
}

impl UploadStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_upload(&self, upload_id: Uuid, file: WebSysFile, total_chunks: i64) {
        let progress = UploadProgress {
            upload_id,
            filename: file.name(),
            size_bytes: file.size() as i64,
            uploaded_bytes: 0,
            status: "pending".to_string(),
            error: None,
        };
        self.uploads.update(|uploads| {
            uploads.insert(upload_id, progress);
        });
    }

    pub fn update_progress(&self, upload_id: Uuid, uploaded: i64, total_chunks: i64) {
        self.uploads.update(|uploads| {
            if let Some(progress) = uploads.get_mut(&upload_id) {
                progress.uploaded_bytes = uploaded;
                progress.status = format!("Uploading chunk {} of {}", uploaded, total_chunks);
            }
        });
    }

    pub fn remove_upload(&self, upload_id: Uuid) {
        self.uploads.update(|uploads| {
            uploads.remove(&upload_id);
        });
    }
}

thread_local! {
    pub static AUTH_STORE: AuthStore = AuthStore::new();
    pub static FILE_STORE: FileStore = FileStore::new();
    pub static UPLOAD_STORE: UploadStore = UploadStore::new();
}
