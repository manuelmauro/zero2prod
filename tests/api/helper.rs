use once_cell::sync::Lazy;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use zero2prod::{
    config::{get_configuration, DatabaseSettings},
    email::EmailClient,
    telemetry::get_subscriber,
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let env_filter = "zero2prod=trace,sqlx=trace,tower_http=trace,axum::rejection=trace";

    if std::env::var("TEST_LOG").is_ok() {
        get_subscriber(env_filter, std::io::stdout).init();
    } else {
        get_subscriber(env_filter, std::io::sink).init();
    };
});

pub struct TestApp {
    pub addr: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&config.database).await;

    let email_client = EmailClient::new(
        config.email_client.base_url,
        config
            .email_client
            .sender_email
            .try_into()
            .expect("The sender email should be valid."),
        config.email_client.authorization_token,
        std::time::Duration::from_secs(1),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("The OS should allocate an available port");
    let port = listener.local_addr().unwrap().port();

    let app = TestApp {
        addr: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool.clone(),
    };

    let _ = tokio::spawn(async move {
        zero2prod::app::serve(listener, connection_pool, email_client)
            .await
            .expect("The server should be running")
    });

    app
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("A postgres connection should be created.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("The database should be created.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("A postgres connection pool should be created.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("The migrations should run without error.");

    connection_pool
}

pub fn get_client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}
