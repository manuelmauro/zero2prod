use std::{io, net::IpAddr};

use axum::{extract::FromRef, http::Request, Router};
use axum_extra::extract::cookie::Key;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::{config::Settings, email::EmailClient};

use self::{session_store::RedisStore, ui::not_found::not_found_page};

mod api;
mod authentication;
mod error;
mod extractor;
mod session_store;
mod ui;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    _cache: Pool<RedisConnectionManager>,
    email_client: EmailClient,
    base_url: String,
    hmac_key: Secret<String>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        Key::from(state.hmac_key.expose_secret().as_bytes())
    }
}

fn app_router() -> Router<AppState> {
    ui::router().nest(
        "/api/v1",
        api::health::router()
            .merge(api::subscription::router())
            .merge(api::newsletter::router())
            .merge(api::user::router()),
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
        )
        .expect("The email client should be available");

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

    pub async fn serve(
        self,
        db: PgPool,
        cache: Pool<RedisConnectionManager>,
    ) -> Result<(), io::Error> {
        let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
            let id = uuid::Uuid::new_v4();
            tracing::info_span!(
                "request",
                method = ?request.method(),
                uri = ?request.uri(),
                %id,
            )
        });

        let session_store = RedisStore::new(cache.clone());
        let session_layer = SessionManagerLayer::new(session_store)
            // TODO use config for secure flag
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::minutes(10)));

        let app = app_router()
            .with_state(AppState {
                db,
                _cache: cache,
                email_client: self.email_client,
                base_url: self.base_url,
                hmac_key: self.hmac_key.clone(),
            })
            .layer(session_layer)
            .layer(trace_layer);

        let app = app.fallback(not_found_page);

        axum::serve(self.listener, app.into_make_service()).await
    }
}
