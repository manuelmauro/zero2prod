use super::AppState;
use axum::{routing::get, Router};

pub mod route;

pub fn router() -> Router<AppState> {
    Router::new().route("/app", get(route::admin_dashboard))
}
