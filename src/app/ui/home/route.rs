use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate;

#[tracing::instrument(name = "Home page")]
pub async fn home_page() -> impl IntoResponse {
    HomeTemplate
}
