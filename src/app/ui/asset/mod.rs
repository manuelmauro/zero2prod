use super::AppState;
use axum::Router;
use tower_http::services::ServeDir;

pub fn router() -> Router<AppState> {
    Router::new().nest_service("/assets", ServeDir::new("assets"))
}
