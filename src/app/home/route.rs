use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate;

pub async fn home() -> impl IntoResponse {
    HomeTemplate
}
