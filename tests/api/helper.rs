use std::{env, io};

use bb8_redis::RedisConnectionManager;
use once_cell::sync::Lazy;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    app::App,
    config::{get_configuration, DatabaseSettings},
    telemetry::get_subscriber,
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let env_filter = "zero2prod=trace,sqlx=trace,tower_http=trace,axum::rejection=trace";

    if env::var("TEST_LOG").is_ok() {
        get_subscriber(env_filter, io::stdout).init();
    } else {
        get_subscriber(env_filter, io::sink).init();
    };
});

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}
pub struct TestApp {
    pub addr: String,
    pub db_pool: PgPool,
    pub http_client: ClientWithMiddleware,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn health_check(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/api/v1/health_check", &self.addr))
            .send()
            .await
            .expect("the request should succeed")
    }

    pub async fn post_subscriptions(&self, body: serde_json::Value) -> reqwest::Response {
        self.http_client
            .post(format!("{}/api/v1/subscriptions", &self.addr))
            .json(&body)
            .send()
            .await
            .expect("the request should succeed")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/api/v1/newsletters", &self.addr))
            .json(&body)
            .send()
            .await
            .expect("the request should succeed")
    }

    /// Extract the confirmation links embedded in the request to the email API.
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value =
            serde_json::from_slice(&email_request.body).expect("body should be a valid json");

        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link =
                reqwest::Url::parse(&raw_link).expect("confirmation links should be present");

            // Let's make sure we don't call random APIs on the web
            assert_eq!(
                confirmation_link
                    .host_str()
                    .expect("host string should be available"),
                "127.0.0.1"
            );
            confirmation_link
                .set_port(Some(self.port))
                .expect("link's port should be updated");
            confirmation_link
        };

        let html = get_link(
            body["HtmlBody"]
                .as_str()
                .expect("html body should convert to a string"),
        );
        let plain_text = get_link(
            body["TextBody"]
                .as_str()
                .expect("text body should be a string"),
        );
        ConfirmationLinks { html, plain_text }
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
    let mut config = get_configuration().expect("the configuration should be available");
    config.application.port = 0;
    config.database.database_name = Uuid::new_v4().to_string();
    config.email_client.base_url = email_server.uri();

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let http_client = ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let db = configure_database(&config.database).await;
    let cache = configure_cache(&config.redis_uri.expose_secret()).await;
    let app = App::with(config).await;

    let test_app = TestApp {
        addr: format!("http://127.0.0.1:{}", app.port()),
        db_pool: db.clone(),
        http_client,
        email_server,
        port: app.port(),
    };

    tokio::spawn(async move {
        app.serve(db, cache)
            .await
            .expect("the server should be running")
    });

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("a postgres connection should be created");

    connection
        .execute(format!(r#"create database "{}";"#, config.database_name).as_str())
        .await
        .expect("the database should be created");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("a postgres connection pool should be created");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("the migrations should run without error");

    connection_pool
}

async fn configure_cache(redis_uri: &str) -> bb8::Pool<RedisConnectionManager> {
    let manager =
        bb8_redis::RedisConnectionManager::new(redis_uri).expect("redis uri should be valid");

    bb8::Pool::builder()
        .build(manager)
        .await
        .expect("redis connection pool should be created")
}
