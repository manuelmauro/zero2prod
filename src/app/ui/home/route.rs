use askama::Template;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

const COUNTER_KEY: &str = "counter";

#[derive(Default, Deserialize, Serialize)]
struct Counter(usize);

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate;

#[tracing::instrument(name = "Home page")]
pub async fn home_page(session: Session) -> impl IntoResponse {
    let counter: Counter = session.get(COUNTER_KEY).await.unwrap().unwrap_or_default();
    session.insert(COUNTER_KEY, counter.0 + 1).await.unwrap();
    tracing::info!("Current count: {}", counter.0);

    HomeTemplate
}
