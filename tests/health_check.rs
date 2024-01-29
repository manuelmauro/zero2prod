use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;

#[tokio::test]
async fn health_check_works() {
    let _ =
        tokio::spawn(async move { zero2prod::serve().await.expect("Server should be running") });

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(TracingMiddleware::default())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let response = client
        .get("http://127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Request should succeed");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
