#[derive(serde::Serialize)]
pub struct Error {
    pub code: u16,
    pub message: String,
    pub details: Option<Vec<ErrorDetails>>,
}

#[derive(serde::Serialize)]
pub struct ErrorDetails {
    pub field: String,
    pub message: String,
}
