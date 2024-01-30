use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use zero2prod::config::get_configuration;

struct TestApp {
    addr: String,
    db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let config = get_configuration().expect("Failed to read configuration.");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("The OS should allocate an available port");
    let port = listener.local_addr().unwrap().port();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database.connection_string())
        .await
        .unwrap();

    let app = TestApp {
        addr: format!("http://127.0.0.1:{}", port),
        db_pool: db.clone(),
    };

    let _ = tokio::spawn(async move {
        zero2prod::app::serve(listener, db)
            .await
            .expect("The server should be running")
    });

    app
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
async fn subscribe_returns_a_400_when_data_is_missing() {
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
            400,
            response.status().as_u16(),
            "The API did not fail with 400 when the payload was {}",
            error_message
        )
    }
}
