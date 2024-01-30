use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health_check", get(health_check))
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
