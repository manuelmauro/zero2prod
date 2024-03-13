use crate::app::AppState;
use axum::{routing::post, Router};

pub mod route;
pub mod schema;

pub fn router() -> Router<AppState> {
    // TODO improve module naming
    Router::new().route("/newsletters", post(route::publish_newsletter))
}
