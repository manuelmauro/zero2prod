use self::schema::{
    CreateUserRequestBody, CreateUserResponseBody, LoginUserRequestBody, LoginUserResponseBody,
    WhoamiResponseBody,
};
use self::utils::{hash_password, verify_password};
use crate::app::extractor::AuthUser;
use anyhow::Context;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use super::error::AppResult;
use super::AppState;

pub mod schema;
pub mod utils;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/login", post(login_user))
        .route("/whoami", get(get_current_user))
}

#[tracing::instrument(name = "Create new user", skip(state, body))]
async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequestBody>,
) -> AppResult<Json<CreateUserResponseBody>> {
    let password_hash = hash_password(body.password).await?;

    let user_id = sqlx::query_scalar!(
        r#"insert into "users" (user_id, username, password_hash) values ($1, $2, $3) returning user_id"#,
        uuid::Uuid::new_v4(),
        body.username,
        password_hash
    )
    .fetch_one(&state.db)
    .await
    .context("User already exist.")?;

    Ok(Json(CreateUserResponseBody {
        token: AuthUser { user_id }.to_jwt(&state),
    }))
}

#[tracing::instrument(name = "Login", skip(state, body))]
async fn login_user(
    State(state): State<AppState>,
    Json(body): Json<LoginUserRequestBody>,
) -> AppResult<Json<LoginUserResponseBody>> {
    let user = sqlx::query!(
        r#"
            select user_id, username, password_hash 
            from "users" where username = $1
        "#,
        body.username,
    )
    .fetch_one(&state.db)
    .await
    .context("No user with such username.")?;

    verify_password(body.password, user.password_hash).await?;

    Ok(Json(LoginUserResponseBody {
        token: AuthUser {
            user_id: user.user_id,
        }
        .to_jwt(&state),
    }))
}

#[tracing::instrument(name = "Whoami", skip(auth_user, state))]
async fn get_current_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<WhoamiResponseBody>> {
    let user = sqlx::query!(
        r#"select username from "users" where user_id = $1"#,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await
    .context("Cannot find user.")?;

    Ok(Json(WhoamiResponseBody {
        username: user.username,
    }))
}
