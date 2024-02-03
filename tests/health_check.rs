use once_cell::sync::Lazy;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use secrecy::ExposeSecret;
use serde_json::Value;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use zero2prod::{
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

struct TestApp {
    addr: String,
    db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&config.database).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("The OS should allocate an available port");
    let port = listener.local_addr().unwrap().port();

    let app = TestApp {
        addr: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool.clone(),
    };

    let _ = tokio::spawn(async move {
        zero2prod::app::serve(listener, connection_pool)
            .await
            .expect("The server should be running")
    });

    app
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

fn get_client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = get_client();

    let response = client
        .get(format!("{}/health_check", app.addr))
        .send()
        .await
        .expect("Request should succeed");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = get_client();
    let body = r#"{"name": "bulbasaur", "email": "bulbasaur@mail.com"}"#;

    let response = client
        .post(format!("{}/subscribe", app.addr))
        .json(&serde_json::from_str::<Value>(body).unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "bulbasaur@mail.com");
    assert_eq!(saved.name, "bulbasaur");
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let app = spawn_app().await;
    let client = get_client();
    let test_cases = [
        (r#"{"name": "bulbasaur"}"#, "missing the email"),
        (r#"{"email": "bulbasaur@mail.com"}"#, "missing the name"),
        ("{}", "missing both name and email"),
    ];

    for (invald_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscribe", app.addr))
            .json(&serde_json::from_str::<Value>(invald_body).unwrap())
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 when the payload was {}",
            error_message
        )
    }
}
