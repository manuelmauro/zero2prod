use anyhow::Context;
use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;

use super::schema;
use crate::{
    app::{error::AppResult, AppState},
    domain::subscriber::email::Email,
};

#[tracing::instrument(name = "Publish newsletter", skip(state, body))]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    Json(body): Json<schema::PublishNewsletterRequestBody>,
) -> AppResult<StatusCode> {
    let subscribers = get_confirmed_subscribers(&state.db)
        .await
        .context("Failed to retrieve confirmed subscribers.")?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        body.title.as_str(),
                        body.content.html.as_str(),
                        body.content.text.as_str(),
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}.", subscriber.email)
                    })?;
            }
            Err(e) => {
                tracing::warn!(
                    details = ?e,
                    "Skipping a confirmed subscriber. Their stored contact details are invalid."
                );
            }
        }
    }

    Ok(StatusCode::OK)
}

struct ConfirmedSubscriber {
    email: Email,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match r.email.try_into() {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(e) => Err(anyhow::anyhow!(e)),
    })
    .collect();

    Ok(confirmed_subscribers)
}
