use anyhow::Context;
use askama::Template;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use uuid::Uuid;

use crate::app::{
    authentication::{self, validate_credentials, Credentials},
    extractor::session_user::SessionUser,
    AppState,
};

use super::schema::ChangePasswordRequestBody;

#[derive(Template)]
#[template(path = "incorrect_username_or_password.html")]
struct IncorrectPasswordTemplate;

#[derive(Template)]
#[template(path = "success.html")]
struct Success {
    message: String,
}

#[derive(Template)]
#[template(path = "admin_dashboard.html")]
struct AdminDashboardTemplate {
    user: String,
}

#[tracing::instrument(name = "Admin dashboard", skip(state, session))]
pub async fn admin_dashboard(
    state: State<AppState>,
    session: Option<SessionUser>,
) -> impl IntoResponse {
    if let Some(user) = session {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                AdminDashboardTemplate {
                    user: get_username(user.id, &state.db).await.unwrap(),
                }
                .render()
                .unwrap(),
            ))
            .unwrap()
    } else {
        Redirect::temporary("/login").into_response()
    }
}

#[tracing::instrument(name = "Change password", skip(user, state, body))]
pub async fn change_password(
    user: SessionUser,
    state: State<AppState>,
    Json(body): Json<ChangePasswordRequestBody>,
) -> impl IntoResponse {
    if body.new_password.expose_secret() != body.new_password_check.expose_secret() {
        return Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(IncorrectPasswordTemplate.render().unwrap()))
            .unwrap();
    }

    let username = get_username(user.id, &state.db).await.unwrap();
    let credentials = Credentials {
        username,
        password: body.current_password,
    };
    if validate_credentials(credentials, &state.db).await.is_err() {
        return Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(IncorrectPasswordTemplate.render().unwrap()))
            .unwrap();
    }

    authentication::change_password(user.id, body.new_password, &state.db)
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            Success {
                message: "Password successfully changed.".to_owned(),
            }
            .render()
            .unwrap(),
        ))
        .unwrap()
}

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        select username
        from users
        where user_id = $1
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}
