use askama::Template;
use axum::{http::StatusCode, response::IntoResponse};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

pub async fn login_form() -> impl IntoResponse {
    LoginTemplate
}

pub async fn login() -> impl IntoResponse {
    StatusCode::OK
}
