use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(serde::Deserialize)]
pub struct NewSubscriber {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(Json(_body): Json<NewSubscriber>) -> impl IntoResponse {
    StatusCode::OK
}
