use crate::helper::{get_client, spawn_app};

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
