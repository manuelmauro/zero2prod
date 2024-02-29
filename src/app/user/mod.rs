use self::schema::{LoginUser, NewUser, User};
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

#[tracing::instrument(name = "Register new user", skip(state, body))]
async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<NewUser>,
) -> AppResult<Json<User>> {
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

    Ok(Json(User {
        token: AuthUser { user_id }.to_jwt(&state),
        username: body.username,
    }))
}

#[tracing::instrument(name = "Login", skip(state, body))]
async fn login_user(
    State(state): State<AppState>,
    Json(body): Json<LoginUser>,
) -> AppResult<Json<User>> {
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

    Ok(Json(User {
        token: AuthUser {
            user_id: user.user_id,
        }
        .to_jwt(&state),
        username: user.username,
    }))
}

#[tracing::instrument(name = "Whoami", skip(auth_user, state))]
async fn get_current_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<User>> {
    let user = sqlx::query!(
        r#"select username from "users" where user_id = $1"#,
        auth_user.user_id
    )
    .fetch_one(&state.db)
    .await
    .context("Cannot find user.")?;

    Ok(Json(User {
        token: auth_user.to_jwt(&state),
        username: user.username,
    }))
}
