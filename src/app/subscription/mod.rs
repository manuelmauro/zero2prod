use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use tracing::instrument;

use crate::domain::subscriber;

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/subscribe", post(subscribe))
}

#[derive(serde::Deserialize)]
pub struct NewSubscriber {
    pub name: subscriber::Name,
    pub email: String,
}

#[instrument(name = "adding a new subscriber", skip(state, body), fields(email = %body.email, name = %body.name))]
pub async fn subscribe(
    State(state): State<AppState>,
    Json(body): Json<NewSubscriber>,
) -> impl IntoResponse {
    match insert_subscriber(&state.db, body).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[instrument(name = "inserting new subscriber into the database", skip(db, subscriber), fields(email = %subscriber.email, name = %subscriber.name))]
async fn insert_subscriber(
    db: &sqlx::PgPool,
    subscriber: NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into subscriptions (id, email, name, subscribed_at) values ($1, $2, $3, $4) returning id"#,
        uuid::Uuid::new_v4(),
        subscriber.email,
        subscriber.name.as_ref(),
        chrono::Utc::now(),
    ).fetch_one(db).await.map_err(|e| {
        tracing::error!(detail = e.to_string(), "failed to save new subscriber");
        e
    })?;

    Ok(())
}
