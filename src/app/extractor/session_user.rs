use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;

use crate::app::error::AppError;

const USER_ID: &str = "user_id";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SessionUser {
    pub id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state)
            .await
            .map_err(|e| AppError::Authorization(e.1.to_owned()))?;
        let user_id: Uuid = session
            .get(USER_ID)
            .await
            .ok()
            .flatten()
            .ok_or_else(|| AppError::Authorization("User not in session".to_owned()))?;

        Ok(SessionUser { id: user_id })
    }
}
