use anyhow::Context;
use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;

use super::schema;
use crate::{
    app::{error::AppResult, AppState},
    domain::subscriber::email::Email,
};

pub async fn publish_newsletter(
    State(state): State<AppState>,
    Json(body): Json<schema::PublishNewsletterRequestBody>,
) -> AppResult<StatusCode> {
    let subscribers = get_confirmed_subscribers(&state.db)
        .await
        .context("Failed to retrieve confirmed subscribers.")?;

    for subscriber in subscribers {
        state
            .email_client
            .send_email(
                &subscriber.email,
                body.title.as_str(),
                body.content.html.as_str(),
                body.content.text.as_str(),
            )
            .await
            .with_context(|| format!("Failed to send newsletter issue to {}", subscriber.email))?;
    }

    Ok(StatusCode::OK)
}

struct ConfirmedSubscriber {
    email: Email,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    struct Row {
        email: String,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .filter_map(|r| match r.email.try_into() {
            Ok(email) => Some(ConfirmedSubscriber { email }),
            Err(e) => {
                tracing::warn!(
                    "A confirmed subscriber is using an invalid email address.\n{}.",
                    e
                );
                None
            }
        })
        .collect();

    Ok(confirmed_subscribers)
}
