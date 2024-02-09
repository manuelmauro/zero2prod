use super::AppState;
use crate::{domain::subscriber::NewSubscriber, email::EmailClient};
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

    if let Err(_) = insert_subscriber(&state.db, &new_subscriber).await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if let Err(_) = send_confirmation_email(&state.email_client, &new_subscriber).await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

#[instrument(name = "inserting new subscriber into the database", skip(db, subscriber), fields(email = %subscriber.email, name = %subscriber.name))]
async fn insert_subscriber(
    db: &sqlx::PgPool,
    subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into subscriptions (id, email, name, subscribed_at, status) values ($1, $2, $3, $4, 'pending_confirmation') returning id"#,
        uuid::Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        chrono::Utc::now(),
    ).fetch_one(db).await.map_err(|e| {
        tracing::error!(detail = e.to_string(), "failed to save new subscriber");
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: &NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://there-is-no-such-domain.com/subscriptions/confirm";
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(&new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}
