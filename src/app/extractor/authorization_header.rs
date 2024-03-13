use anyhow::{anyhow, Context};
use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use secrecy::ExposeSecret;
use sha2::Sha384;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app::error::AppError;
use crate::app::AppState;

const DEFAULT_SESSION_LENGTH: time::Duration = time::Duration::weeks(2);

// Create alias for HMAC-SHA384
type HmacSha384 = Hmac<Sha384>;

/// Add this as a parameter to a handler function to require the user to be logged in.
///
/// Parses a JWT from the `Authorization: Token <token>` header.
pub struct ApiToken {
    pub user_id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    user_id: Uuid,
    /// Standard JWT `exp` claim.
    exp: i64,
}

impl ApiToken {
    pub(in crate::app) fn to_jwt(&self, state: &AppState) -> String {
        let hmac = HmacSha384::new_from_slice(state.hmac_key.expose_secret().as_bytes())
            .expect("HMAC-SHA-384 should accept any key length");

        Claims {
            user_id: self.user_id,
            exp: (OffsetDateTime::now_utc() + DEFAULT_SESSION_LENGTH).unix_timestamp(),
        }
        .sign_with_key(&hmac)
        .expect("HMAC signing should be infallible")
    }

    /// Attempt to parse `Self`.
    pub fn from_str(state: &AppState, token: &str) -> anyhow::Result<Self> {
        let jwt = jwt::Token::<jwt::Header, Claims, _>::parse_unverified(token)
            .context("Failed to parse the JWT token")?;

        let hmac = HmacSha384::new_from_slice(state.hmac_key.expose_secret().as_bytes())
            .expect("HMAC-SHA-384 should accept any key length");

        let jwt = jwt.verify_with_key(&hmac)?;

        let (_header, claims) = jwt.into();

        if claims.exp < OffsetDateTime::now_utc().unix_timestamp() {
            return Err(anyhow!("Token expired"));
        }

        Ok(Self {
            user_id: claims.user_id,
        })
    }
}

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
