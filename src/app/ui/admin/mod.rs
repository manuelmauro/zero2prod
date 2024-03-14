use super::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub mod route;
pub mod schema;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/app", get(route::admin_dashboard))
        .route("/change-password", post(route::change_password))
}
