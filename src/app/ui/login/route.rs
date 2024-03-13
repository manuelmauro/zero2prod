use askama::Template;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use secrecy::Secret;

use super::schema;
use crate::app::{
    api::user::auth::{validate_credentials, Credentials},
    extractor::authorization_header::ApiToken,
    AppState,
};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "incorrect_username_or_password.html")]
struct IncorrectUsernameOrPasswordTemplate;

#[tracing::instrument(name = "Login form")]
pub async fn login_form() -> impl IntoResponse {
    LoginTemplate
}

#[tracing::instrument(
    name = "Login",
    skip(jar, state, body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    Json(body): Json<schema::LoginRequestBody>,
) -> (PrivateCookieJar, impl IntoResponse) {
    let credentials = Credentials {
        username: body.username,
        password: Secret::new(body.password),
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &state.db).await {
        Ok(user_id) => {
            let updated_jar = jar.add(Cookie::new("session", ApiToken { user_id }.to_jwt(&state)));
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            (
                updated_jar,
                Response::builder()
                    .status(StatusCode::OK)
                    .header("HX-Redirect", "/app")
                    .body(Body::empty())
                    .unwrap(),
            )
        }
        Err(_) => (
            jar,
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(
                    IncorrectUsernameOrPasswordTemplate.render().unwrap(),
                ))
                .unwrap(),
        ),
    }
}
