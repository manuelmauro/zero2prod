use askama::Template;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};

use super::schema;
use crate::app::AppState;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

pub async fn login_form() -> impl IntoResponse {
    LoginTemplate
}

#[tracing::instrument(name = "Login", skip(_state, _body))]
pub async fn login(
    State(_state): State<AppState>,
    Json(_body): Json<schema::LoginRequestBody>,
) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("HX-Redirect", "/")
        .body(Body::empty())
        .unwrap()
}
