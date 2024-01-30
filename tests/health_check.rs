use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde_json::Value;
use sqlx::{Connection, PgConnection};
use zero2prod::config::get_configuration;

async fn spawn_app() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("The OS should allocate an available port");
    let port = listener.local_addr().unwrap().port();

    let _ = tokio::spawn(async move {
        zero2prod::app::serve(listener)
            .await
            .expect("The server should be running")
    });

    format!("http://127.0.0.1:{}", port)
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
    let addr = spawn_app().await;
    let client = get_client();

    let response = client
        .get(format!("{}/health_check", addr))
        .send()
        .await
        .expect("Request should succeed");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let addr = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    // The `Connection` trait MUST be in scope for us to invoke
    // `PgConnection::connect` - it is not an inherent method of the struct!
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let client = get_client();
    let body = r#"{"name": "bulbasaur", "email": "bulbasaur@mail.com"}"#;

    let response = client
        .post(format!("{}/subscribe", addr))
        .json(&serde_json::from_str::<Value>(body).unwrap())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "bulbasaur@mail.com");
    assert_eq!(saved.name, "bulbasaur");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let addr = spawn_app().await;
    let client = get_client();
    let test_cases = [
        (r#"{"name": "bulbasaur"}"#, "missing the email"),
        (r#"{"email": "bulbasaur@mail.com"}"#, "missing the name"),
        ("{}", "missing both name and email"),
    ];

    for (invald_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscribe", addr))
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
