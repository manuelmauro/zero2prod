use super::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub mod route;
pub mod schema;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(route::login_form))
        .route("/login", post(route::login))
}
