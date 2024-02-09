use once_cell::sync::Lazy;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use zero2prod::{
    app::App,
    config::{get_configuration, DatabaseSettings},
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
    config.application.port = 0;
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&config.database).await;

    let app = App::with(config).await;

    let test_app = TestApp {
        addr: format!("http://127.0.0.1:{}", app.port()),
        db_pool: connection_pool.clone(),
    };

    let _ = tokio::spawn(async move {
        app.serve(connection_pool)
            .await
            .expect("The server should be running")
    });

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
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
