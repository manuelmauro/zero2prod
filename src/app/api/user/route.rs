use anyhow::Context;
use axum::extract::State;
use axum::Json;
use secrecy::{ExposeSecret, Secret};

use super::schema::{
    CreateUserRequestBody, CreateUserResponseBody, LoginUserRequestBody, LoginUserResponseBody,
    WhoamiResponseBody,
};
use super::AppState;

use crate::app::authentication::{compute_password_hash, validate_credentials, Credentials};
use crate::app::error::{AppError, AppResult};
use crate::app::extractor::authorization_header::ApiToken;
use crate::telemetry::spawn_blocking_with_tracing;

#[tracing::instrument(name = "Create new user", skip(state, body))]
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequestBody>,
) -> AppResult<Json<CreateUserResponseBody>> {
    let password_hash =
        spawn_blocking_with_tracing(move || compute_password_hash(Secret::new(body.password)))
            .await
            .context("Could not compute password hash.")??;

    let user_id = sqlx::query_scalar!(
        r#"insert into "users" (user_id, username, password_hash) values ($1, $2, $3) returning user_id"#,
        uuid::Uuid::new_v4(),
        body.username,
        password_hash.expose_secret()
    )
    .fetch_one(&state.db)
    .await
    .context("User already exist.")?;

    Ok(Json(CreateUserResponseBody {
        token: ApiToken { user_id }.to_jwt(&state),
    }))
}

#[tracing::instrument(skip(state, body), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login_user(
    State(state): State<AppState>,
    Json(body): Json<LoginUserRequestBody>,
) -> AppResult<Json<LoginUserResponseBody>> {
    let credentials = Credentials {
        username: body.username,
        password: Secret::new(body.password),
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &state.db).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            Ok(Json(LoginUserResponseBody {
                token: ApiToken { user_id }.to_jwt(&state),
            }))
        }
        Err(e) => Err(AppError::Unexpected(e.into())),
    }
}

#[tracing::instrument(name = "Whoami", skip(token, state))]
pub async fn get_current_user(
    token: ApiToken,
    State(state): State<AppState>,
) -> AppResult<Json<WhoamiResponseBody>> {
    let user = sqlx::query!(
        r#"select username from "users" where user_id = $1"#,
        token.user_id
    )
    .fetch_one(&state.db)
    .await
    .context("User does not exists.")?;

    Ok(Json(WhoamiResponseBody {
        username: user.username,
    }))
}
