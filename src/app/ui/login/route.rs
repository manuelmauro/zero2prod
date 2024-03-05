use askama::Template;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use secrecy::Secret;

use super::schema;
use crate::app::{
    user::auth::{validate_credentials, Credentials},
    AppState,
};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "incorrect_username_or_password.html")]
struct IncorrectUsernameOrPasswordTemplate;

pub async fn login_form() -> impl IntoResponse {
    LoginTemplate
}

#[tracing::instrument(
    name = "Login",
    skip(state, body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<schema::LoginRequestBody>,
) -> impl IntoResponse {
    let credentials = Credentials {
        username: body.username,
        password: Secret::new(body.password),
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &state.db).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            Response::builder()
                .status(StatusCode::OK)
                .header("HX-Redirect", "/admin/dashboard")
                .body(Body::empty())
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                IncorrectUsernameOrPasswordTemplate.render().unwrap(),
            ))
            .unwrap(),
    }
}
