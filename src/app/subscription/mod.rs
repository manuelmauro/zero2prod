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
    if let Ok(id) = sqlx::query_scalar!(
        r#"insert into subscriptions (id, email, name, subscribed_at) values ($1, $2, $3, $4) returning id"#,
        uuid::Uuid::new_v4(),
        body.email,
        body.name,
        chrono::Utc::now(),

    ).fetch_one(&state.db).await {
        return Ok(id.to_string());
    };

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
