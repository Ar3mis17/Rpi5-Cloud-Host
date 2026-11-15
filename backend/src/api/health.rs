use axum::{
    extract::State,
    Json,
};
use sqlx::PgPool;
use crate::db::health_check;
use shared::HealthResponse;

pub async fn health_check_handler(
    State((pool, _)): State<(PgPool, std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, crate::models::UploadSession>>>)>,
) -> Json<HealthResponse> {
    let db_ok = health_check(&pool).await;
    Json(HealthResponse {
        status: if db_ok { "healthy" } else { "unhealthy" }.to_string(),
        service: "ar3mis-cloud-backend".to_string(),
        database: if db_ok { "connected" } else { "disconnected" }.to_string(),
        timestamp: chrono::Utc::now(),
    })
}

pub async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "Ar3mis Cloud",
        "version": "0.1.0",
        "status": "running"
    }))
}
