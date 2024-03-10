use std::{io, net::IpAddr};

use axum::{extract::FromRef, http::Request, Router};
use axum_extra::extract::cookie::Key;
use secrecy::Secret;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{config::Settings, email::EmailClient};

mod api;
mod error;
mod extractor;
mod health;
mod newsletter;
mod subscription;
mod ui;
mod user;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_key: Secret<String>,
    cookie_key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

fn app_router() -> Router<AppState> {
    ui::router().nest(
        "/api/v1",
        health::router()
            .merge(subscription::router())
            .merge(newsletter::router())
            .merge(user::router()),
    )
}

pub struct App {
    listener: TcpListener,
    email_client: EmailClient,
    base_url: String,
    hmac_key: Secret<String>,
}

impl App {
    pub async fn with(config: Settings) -> Self {
        // TODO do not take ownership of the config
        let timeout = config.email_client.timeout();
        let email_client = EmailClient::new(
            config.email_client.base_url,
            config
                .email_client
                .sender_email
                .try_into()
                .expect("The sender email should be valid."),
            config.email_client.authorization_token,
            timeout,
        );

        let listener = tokio::net::TcpListener::bind(format!(
            "{}:{}",
            config.application.host, config.application.port
        ))
        .await
        .expect("The listener should be able to bind the address.");

        Self {
            listener,
            email_client,
            base_url: config.application.base_url,
            hmac_key: config.application.hmac_key,
        }
    }

    pub fn host(&self) -> IpAddr {
        self.listener.local_addr().unwrap().ip()
    }

    pub fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    pub async fn serve(self, db: PgPool) -> Result<(), io::Error> {
        let app = app_router()
            .with_state(AppState {
                db,
                email_client: self.email_client,
                base_url: self.base_url,
                hmac_key: self.hmac_key,
                // TODO add to configuration
                cookie_key: Key::generate(),
            })
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                    let id = uuid::Uuid::new_v4();
                    tracing::info_span!(
                        "request",
                        method = ?request.method(),
                        uri = ?request.uri(),
                        %id,
                    )
                }),
            );

        axum::serve(self.listener, app.into_make_service()).await
    }
}
