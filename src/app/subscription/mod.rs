use super::AppState;
use crate::domain::subscriber::NewSubscriber;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use tracing::instrument;

pub mod schema;

pub fn router() -> Router<AppState> {
    Router::new().route("/subscribe", post(subscribe))
}

#[instrument(name = "adding a new subscriber", skip(state, body), fields(email = %body.email, name = %body.name))]
pub async fn subscribe(
    State(state): State<AppState>,
    Json(body): Json<schema::SubscribeBody>,
) -> impl IntoResponse {
    let new_subscriber = match NewSubscriber::try_from(body) {
        Ok(subscriber) => subscriber,
        Err(e) => {
            tracing::error!(detail = e, "failed to parse subscriber from body");
            return StatusCode::BAD_REQUEST;
        }
    };

    match insert_subscriber(&state.db, new_subscriber).await {
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
        r#"insert into subscriptions (id, email, name, subscribed_at, status) values ($1, $2, $3, $4, $5) returning id"#,
        uuid::Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        chrono::Utc::now(),
        "confirmed",
    ).fetch_one(db).await.map_err(|e| {
        tracing::error!(detail = e.to_string(), "failed to save new subscriber");
        e
    })?;

    Ok(())
}
