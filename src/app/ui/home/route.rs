use askama::Template;
use axum::response::IntoResponse;
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate;

#[tracing::instrument(name = "Home page")]
pub async fn home_page(session: Session) -> impl IntoResponse {
    HomeTemplate
}
