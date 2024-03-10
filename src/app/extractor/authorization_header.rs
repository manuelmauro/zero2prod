use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;

use crate::app::api::token::ApiToken;
use crate::app::error::AppError;
use crate::app::AppState;

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
        let auth_header: TypedHeader<Authorization<Bearer>> =
            TypedHeader::from_request_parts(parts, &state)
                .await
                .map_err(|_| AppError::Authorization("Missing Authorization header.".to_owned()))?;

        Self::from_str(&state, auth_header.token())
            .map_err(|_| AppError::Authorization("Invalid Authorization header.".to_owned()))
    }
}
