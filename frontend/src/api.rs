//! Frontend API client

use gloo::net::http::Request;
use gloo::storage::{LocalStorage, Storage};
use shared::{
    UserResponse, FileEntry, PaginatedResponse, AuthResponse,
    RegisterRequest, LoginRequest, CreateFolderRequest, UpdateFileRequest,
    UploadInitiateRequest, UploadInitiateResponse, UploadStatusResponse, StorageQuota
};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

fn get_token() -> Option<String> {
    LocalStorage::get("auth_token").ok()
}

pub fn set_token(token: &str) {
    let _ = LocalStorage::set("auth_token", token);
}

pub fn set_refresh_token(token: &str) {
    let _ = LocalStorage::set("refresh_token", token);
}

pub fn clear_token() {
    let _ = LocalStorage::delete("auth_token");
    let _ = LocalStorage::delete("refresh_token");
}

async fn request_json<T: serde::de::DeserializeOwned, B: serde::Serialize + ?Sized>(
    method: &str,
    url: &str,
    body: Option<&B>,
) -> Result<T, String> {
    let mut builder = match method {
        "GET" => Request::get(url),
        "POST" => Request::post(url),
        "PUT" => Request::put(url),
        "PATCH" => Request::patch(url),
        "DELETE" => Request::delete(url),
        _ => return Err("Invalid method".to_string()),
    };

    builder = builder.header("Content-Type", "application/json");

    if let Some(token) = get_token() {
        builder = builder.header("Authorization", &format!("Bearer {}", token));
    }

    let response = if let Some(b) = body {
        builder.json(b).map_err(|e| e.to_string())?
            .send().await
    } else {
        builder.send().await
    }.map_err(|e| e.to_string())?;

    if !response.ok() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", response.status(), text));
    }

    response.json::<T>().await.map_err(|e| e.to_string())
}

pub async fn register(username: String, email: String, password: String) -> Result<AuthResponse, String> {
    let req = RegisterRequest { username, email, password };
    let res: AuthResponse = request_json("POST", "/api/v1/auth/register", Some(&req)).await?;
    set_token(&res.tokens.access_token);
    set_refresh_token(&res.tokens.refresh_token);
    Ok(res)
}

pub async fn login(email: String, password: String) -> Result<AuthResponse, String> {
    let req = LoginRequest { email, password, remember_me: Some(true) };
    let res: AuthResponse = request_json("POST", "/api/v1/auth/login", Some(&req)).await?;
    set_token(&res.tokens.access_token);
    set_refresh_token(&res.tokens.refresh_token);
    Ok(res)
}

pub async fn refresh_token() -> Result<AuthResponse, String> {
    let refresh_token: Result<String, StorageError> = LocalStorage::get("refresh_token");
    let refresh_token = refresh_token.map_err(|_| "No refresh token".to_string())?;
    let req = serde_json::json!({ "refresh_token": refresh_token });
    let res: AuthResponse = request_json("POST", "/api/v1/auth/refresh", Some(&req)).await?;
    set_token(&res.tokens.access_token);
    set_refresh_token(&res.tokens.refresh_token);
    Ok(res)
}

pub async fn logout() -> Result<(), String> {
    let _: serde_json::Value = request_json("POST", "/api/v1/auth/logout", None).await?;
    clear_token();
    Ok(())
}

pub async fn get_me() -> Result<UserResponse, String> {
    request_json("GET", "/api/v1/auth/me", None).await
}

pub async fn list_files(parent_id: Option<i64>, page: i64, page_size: i64) -> Result<PaginatedResponse<FileEntry>, String> {
    let mut url = format!("/api/v1/files?page={}&page_size={}", page, page_size);
    if let Some(pid) = parent_id {
        url.push_str(&format!("&parent_id={}", pid));
    }
    request_json("GET", &url, None).await
}

pub async fn create_folder(req: CreateFolderRequest) -> Result<FileEntry, String> {
    request_json("POST", "/api/v1/files/folder", Some(&req)).await
}

pub async fn get_file(id: i64) -> Result<FileEntry, String> {
    request_json("GET", &format!("/api/v1/files/{}", id), None).await
}

pub async fn update_file(id: i64, name: Option<String>, parent_id: Option<i64>) -> Result<FileEntry, String> {
    let req = UpdateFileRequest { name, parent_id };
    request_json("PATCH", &format!("/api/v1/files/{}", id), Some(&req)).await
}

pub async fn delete_file(id: i64) -> Result<(), String> {
    let _: serde_json::Value = request_json("DELETE", &format!("/api/v1/files/{}", id), None).await?;
    Ok(())
}

pub async fn download_file(id: i64) -> Result<String, String> {
    Ok(format!("/api/v1/files/{}/download", id))
}

pub async fn get_storage_quota() -> Result<StorageQuota, String> {
    request_json("GET", "/api/v1/files/quota", None).await
}

pub async fn initiate_upload(req: UploadInitiateRequest) -> Result<UploadInitiateResponse, String> {
    request_json("POST", "/api/v1/uploads/initiate", Some(&req)).await
}

pub async fn upload_chunk(upload_id: Uuid, chunk_number: i32, data: Vec<u8>) -> Result<(), String> {
    let url = format!("/api/v1/uploads/{}/chunk?chunk_number={}", upload_id, chunk_number);
    let mut builder = Request::post(&url);
    if let Some(token) = get_token() {
        builder = builder.header("Authorization", &format!("Bearer {}", token));
    }
    builder = builder.header("Content-Type", "application/octet-stream");

    let response = builder.body(data).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;

    if !response.ok() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Chunk upload error {}: {}", response.status(), text));
    }
    Ok(())
}

pub async fn get_upload_status(upload_id: Uuid) -> Result<UploadStatusResponse, String> {
    request_json("GET", &format!("/api/v1/uploads/{}/status", upload_id), None).await
}

pub async fn complete_upload(upload_id: Uuid) -> Result<FileEntry, String> {
    request_json("POST", &format!("/api/v1/uploads/{}/complete", upload_id), None).await
}
