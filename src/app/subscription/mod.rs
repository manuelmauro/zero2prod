use super::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub mod route;
pub mod schema;

pub fn router() -> Router<AppState> {
    // TODO improve module naming
    Router::new()
        .route("/subscriptions", post(route::subscribe))
        .route("/subscriptions/confirm", get(route::confirm))
}
