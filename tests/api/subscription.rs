use serde_json::Value;

use crate::helper::{get_client, spawn_app};

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
        .expect("The request should succeed.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("The saved subscription should exist.");

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
            .expect("The request should succeed.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 when the payload was {}",
            error_message
        )
    }
}
