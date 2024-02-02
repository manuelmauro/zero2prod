use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use tracing::Instrument;

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
    tracing::info!(
        "adding a new subscriber with email {} and name {} to the database",
        body.email,
        body.name
    );

    let query_span = tracing::info_span!("saving new subscriber details in the database");
    match sqlx::query!(
        r#"insert into subscriptions (id, email, name, subscribed_at) values ($1, $2, $3, $4) returning id"#,
        uuid::Uuid::new_v4(),
        body.email,
        body.name,
        chrono::Utc::now(),
    ).fetch_one(&state.db).instrument(query_span).await {
        Ok(_) => {
            tracing::info!("new subscriber details have been saved");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("failed to save new subscriber details: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR}
            ,
    }
}
