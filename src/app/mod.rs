use std::{io, net::IpAddr};

use axum::{http::Request, Router};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{config::Settings, email::EmailClient};

mod error;
mod health;
mod home;
mod login;
mod newsletter;
mod subscription;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    email_client: EmailClient,
    base_url: String,
}

fn app_router() -> Router<AppState> {
    health::router()
        .merge(subscription::router())
        .merge(newsletter::router())
        .merge(home::router())
        .merge(login::router())
}

pub struct App {
    listener: TcpListener,
    email_client: EmailClient,
    base_url: String,
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
