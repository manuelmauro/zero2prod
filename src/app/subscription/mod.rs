use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/subscribe", post(subscribe))
}

#[derive(serde::Deserialize)]
pub struct NewSubscriber {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(
    State(state): State<AppState>,
    Json(body): Json<NewSubscriber>,
) -> impl IntoResponse {
    let record = sqlx::query!(
        r#"insert into subscriptions (id, email, name, subscribed_at) values ($1, $2, $3, $4) returning id"#,
        uuid::Uuid::new_v4(),
        body.email,
        body.name,
        chrono::Utc::now(),
    ).fetch_one(&state.db).await;

    match record {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
