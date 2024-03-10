use anyhow::{anyhow, Context};
use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::app::api::token::ApiToken;
use crate::app::error::AppError;
use crate::app::AppState;

const SCHEME_PREFIX: &str = "Bearer ";

#[async_trait]
impl FromRequestParts<AppState> for ApiToken
where
    AppState: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let state: State<AppState> = State::from_request_parts(parts, state)
            .await
            .expect("BUG: AppState should be added as an extension");

        // Get the value of the `Authorization` header, if it was sent at all.
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(AppError::Authorization(
                "Missing Authorization header.".to_owned(),
            ))?;

        let auth_header = auth_header
            .to_str()
            .context("Authorization header is not UTF-8")?;

        if !auth_header.starts_with(SCHEME_PREFIX) {
            return Err(AppError::Unexpected(anyhow!(
                "Authorization header is using the wrong scheme"
            )));
        }

        let token = &auth_header[SCHEME_PREFIX.len()..];

        Self::from_str(&state, token)
            .map_err(|_| AppError::Authorization("Invalid Authorization header.".to_owned()))
    }
}
