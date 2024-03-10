use anyhow::{anyhow, Context};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use secrecy::ExposeSecret;
use sha2::Sha384;
use time::OffsetDateTime;
use uuid::Uuid;

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
