use axum::http::StatusCode;

use crate::app::error::AppResult;

pub async fn publish_newsletter() -> AppResult<StatusCode> {
    Ok(StatusCode::OK)
}
