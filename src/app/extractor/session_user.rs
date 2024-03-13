use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{self, request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

const USER_ID: &str = "user_id";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SessionUser {
    pub id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionUser
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;
        let user_id: String = session
            .get(USER_ID)
            .await
            .unwrap()
            .ok_or((StatusCode::UNAUTHORIZED, "User not authenticated"))?;

        Ok(SessionUser { id: user_id })
    }
}
