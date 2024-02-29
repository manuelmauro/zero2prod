use super::AppState;
use axum::routing::{get, post};
use axum::Router;

pub mod route;
pub mod schema;
pub mod utils;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/users", post(route::create_user))
        .route("/users/login", post(route::login_user))
        .route("/whoami", get(route::get_current_user))
}
