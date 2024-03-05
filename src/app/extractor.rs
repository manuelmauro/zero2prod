use anyhow::{anyhow, Context};
use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{request::Parts, HeaderValue};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use secrecy::ExposeSecret;
use sha2::Sha384;
use time::OffsetDateTime;
use uuid::Uuid;

use super::error::AppError;
use super::AppState;

const DEFAULT_SESSION_LENGTH: time::Duration = time::Duration::weeks(2);
const SCHEME_PREFIX: &str = "Bearer ";

// Create alias for HMAC-SHA384
type HmacSha384 = Hmac<Sha384>;

/// Add this as a parameter to a handler function to require the user to be logged in.
///
/// Parses a JWT from the `Authorization: Token <token>` header.
pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthUserClaims {
    user_id: Uuid,
    /// Standard JWT `exp` claim.
    exp: i64,
}

impl AuthUser {
    pub(in crate::app) fn to_jwt(&self, state: &AppState) -> String {
        let hmac = HmacSha384::new_from_slice(state.hmac_key.expose_secret().as_bytes())
            .expect("HMAC-SHA-384 should accept any key length");

        AuthUserClaims {
            user_id: self.user_id,
            exp: (OffsetDateTime::now_utc() + DEFAULT_SESSION_LENGTH).unix_timestamp(),
        }
        .sign_with_key(&hmac)
        .expect("HMAC signing should be infallible")
    }

    /// Attempt to parse `Self` from an `Authorization` header.
    fn from_authorization(state: &AppState, auth_header: &HeaderValue) -> anyhow::Result<Self> {
        let auth_header = auth_header
            .to_str()
            .context("Authorization header is not UTF-8")?;

        if !auth_header.starts_with(SCHEME_PREFIX) {
            return Err(anyhow!("Authorization header is using the wrong scheme"));
        }

        let token = &auth_header[SCHEME_PREFIX.len()..];

        let jwt = jwt::Token::<jwt::Header, AuthUserClaims, _>::parse_unverified(token)
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
impl FromRequestParts<AppState> for AuthUser
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

        Self::from_authorization(&state, auth_header)
            .map_err(|_| AppError::Authorization("Invalid Authorization header.".to_owned()))
    }
}
