use axum::{routing::get, Router};

use crate::app::AppState;

pub mod route;

pub fn router() -> Router<AppState> {
    Router::new().route("/health_check", get(route::health_check))
}
