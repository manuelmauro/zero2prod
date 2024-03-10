use anyhow::{anyhow, Context};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum_extra::extract::cookie::Key;
use axum_extra::extract::PrivateCookieJar;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use secrecy::ExposeSecret;
use sha2::Sha384;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app::error::AppError;
use crate::app::AppState;

// Create alias for HMAC-SHA384
type HmacSha384 = Hmac<Sha384>;

/// Add this as a parameter to a handler function to require the user to be logged in.
///
/// Parses a JWT from the `Authorization: Token <token>` header.
#[derive(Debug)]
pub struct SessionCookie {
    pub user_id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    user_id: Uuid,
    /// Standard JWT `exp` claim.
    exp: i64,
}

impl SessionCookie {
    fn from_str(state: &AppState, token: &str) -> anyhow::Result<Self> {
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
impl FromRequestParts<AppState> for SessionCookie
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

        let cookie_jar: PrivateCookieJar<Key> =
            PrivateCookieJar::from_request_parts(parts, &state.0)
                .await
                .map_err(|_| {
                    axum::response::Response::builder()
                        .status(401)
                        .body(Body::empty())
                        .unwrap()
                })
                .unwrap();

        let token = cookie_jar
            .get("session")
            .ok_or_else(|| {
                axum::response::Response::builder()
                    .status(401)
                    .body(Body::empty())
                    .unwrap()
            })
            .unwrap();

        tracing::debug!("{:?}", token);

        Self::from_str(&state, token.value_trimmed())
            .map_err(|_| AppError::Authorization("Invalid Authorization header.".to_owned()))
    }
}
