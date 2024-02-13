use super::schema::{self, ConfirmParams};
use crate::{app::AppState, domain::subscriber::NewSubscriber, email::EmailClient};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use tracing::instrument;
use uuid::Uuid;

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

    let mut transaction = match state.db.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let subscription_token = generate_subscription_token();
    if let Err(_) = store_token(&mut transaction, subscriber_id, &subscription_token).await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if let Err(_) = transaction.commit().await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if let Err(_) = send_confirmation_email(
        &state.email_client,
        &new_subscriber,
        &state.base_url,
        &subscription_token,
    )
    .await
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, params))]
pub async fn confirm(
    State(state): State<AppState>,
    Query(params): Query<ConfirmParams>,
) -> impl IntoResponse {
    let id = match get_subscriber_id_from_token(&state.db, &params.subscription_token).await {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    match id {
        // Non-existing token!
        None => StatusCode::UNAUTHORIZED,
        Some(subscriber_id) => {
            if confirm_subscriber(&state.db, subscriber_id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
    }
}

#[instrument(name = "inserting new subscriber into the database", skip(transaction, subscriber), fields(email = %subscriber.email, name = %subscriber.name))]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"insert into subscriptions (id, email, name, subscribed_at, status) values ($1, $2, $3, $4, 'pending_confirmation') returning id"#,
        subscriber_id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        chrono::Utc::now(),
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!(detail = e.to_string(), "failed to save new subscriber");
        e
    })?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: &NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );

    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    if let Err(e) = email_client
        .send_email(&new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
    {
        tracing::error!(
            detail = e.to_string(),
            "Failed to send a confirmation email to the new subscriber"
        );
        return Err(e);
    }

    Ok(())
}

/// Generate a random 25-characters-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
      WHERE subscription_token = $1",
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}
