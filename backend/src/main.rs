//! Ar3mis Cloud Backend - Axum server

use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::get_config;
use crate::db::create_pool;
use crate::middleware::auth::{auth_middleware, optional_auth_middleware};
use crate::api::{auth, files, uploads, shares, health};
use crate::errors::AppResult;

mod config;
mod db;
mod models;
mod auth_utils;
mod errors;
mod middleware;
mod services;
mod api;

type UploadSessions = Arc<Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>;

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = create_pool().await?;
    let upload_sessions: UploadSessions = Arc::new(Mutex::new(std::collections::HashMap::new()));

    let cors = CorsLayer::new()
        .allow_origin(get_config().cors_allow_origins.iter().map(|s| s.parse().unwrap()).collect::<Vec<_>>())
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE, axum::http::Method::PATCH])
        .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
        .allow_credentials(true);

    let auth_router = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .route("/logout", post(auth::logout))
        .route("/me", get(auth::me).patch(auth::update_me))
        .route("/change-password", post(auth::change_password))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/sessions", get(auth::list_sessions).delete(auth::revoke_session))
        .layer(axum::middleware::from_fn(auth_middleware));

    let files_router = Router::new()
        .route("/", get(files::list_files))
        .route("/folder", post(files::create_folder))
        .route("/:id", get(files::get_file).patch(files::update_file).delete(files::delete_file))
        .route("/:id/download", get(files::download_file))
        .route("/quota", get(files::get_storage_quota))
        .layer(axum::middleware::from_fn(auth_middleware));

    let uploads_router = Router::new()
        .route("/initiate", post(uploads::initiate_upload))
        .route("/:upload_id/chunk", post(uploads::upload_chunk_with_query))
        .route("/:upload_id/status", get(uploads::get_upload_status))
        .route("/:upload_id/complete", post(uploads::complete_upload))
        .layer(axum::middleware::from_fn(auth_middleware));

    let shares_router = Router::new()
        .route("/", post(shares::create_share).get(shares::list_shares))
        .route("/:share_id", delete(shares::revoke_share))
        .route("/access/:token", get(shares::access_shared_file))
        .layer(axum::middleware::from_fn(auth_middleware));

    let api_router = Router::new()
        .nest("/auth", auth_router)
        .nest("/files", files_router)
        .nest("/uploads", uploads_router)
        .nest("/shares", shares_router)
        .merge(Router::new()
            .route("/health", get(health::health_check_handler))
            .route("/", get(health::root))
            .layer(axum::middleware::from_fn(optional_auth_middleware)));

    let app = Router::new()
        .nest("/api/v1", api_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state((pool, upload_sessions));

    let config = get_config();
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
