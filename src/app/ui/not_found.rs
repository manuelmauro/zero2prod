use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate;

#[tracing::instrument(name = "Not found page")]
pub async fn not_found_page() -> impl IntoResponse {
    NotFoundTemplate
}
