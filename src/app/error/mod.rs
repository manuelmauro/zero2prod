use std::result;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

mod schema;

///
pub type AppResult<T, E = AppError> = result::Result<T, E>;

/// A common error type that can be used throughout the API.
///
/// Can be returned in a `Result` from an API handler function.
///
/// For convenience, this represents both API errors as well as internal recoverable errors,
/// and maps them to appropriate status codes along with at least a minimally useful error
/// message in a plain text body, or a JSON body in the case of `UnprocessableEntity`.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Authorization(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Authorization(_) => StatusCode::UNAUTHORIZED,
            Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Axum allows you to return `Result` from handler functions, but the error type
/// also must be some sort of response type.
///
/// By default, the generated `Display` impl is used to return a plaintext error message
/// to the client.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Unexpected(ref e) => {
                tracing::error!("{:?}", e);
                (
                    self.status_code(),
                    Json(schema::Error {
                        code: 0,
                        message: "Unexpected error".to_owned(),
                        details: None,
                    }),
                )
                    .into_response()
            }
            ref e => {
                tracing::error!("{}", e);
                (self.status_code(), ()).into_response()
            }
        }
    }
}
