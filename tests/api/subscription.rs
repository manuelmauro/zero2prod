use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helper::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = r#"{"name": "bulbasaur", "email": "bulbasaur@mail.com"}"#;
    let response = app.post_subscriptions(body).await;

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
    let test_cases = [
        (r#"{"name": "bulbasaur"}"#, "missing the email"),
        (r#"{"email": "bulbasaur@mail.com"}"#, "missing the name"),
        ("{}", "missing both name and email"),
    ];

    for (invald_body, error_message) in test_cases {
        let response = app.post_subscriptions(invald_body).await;

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 when the payload was {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let test_cases = vec![
        (
            r#"{"name": "", "email": "bulbasaur@mail.com"}"#,
            "empty name",
        ),
        (r#"{"name": "bulbasaur", "email": ""}"#, "empty email"),
        (
            r#"{"name": "bulbasaur", "email": "definitely-not-an-email"}"#,
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscriptions(body).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = r#"{"name": "bulbasaur", "email": "bulbasaur@mail.com"}"#;
    app.post_subscriptions(body.into()).await;
}
