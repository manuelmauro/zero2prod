use std::io;

use bb8_redis::bb8;
use bb8_redis::RedisConnectionManager;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::{app::App, config::get_configuration, telemetry::get_subscriber};

#[tokio::main]
async fn main() {
    let config = get_configuration().expect("The configuration should load.");
    get_subscriber(&config.application.log_level, io::stderr).init();

    tracing::debug!("creating postgres connection pool");
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect_lazy_with(config.database.with_db());

    tracing::debug!("creating redis connection pool");
    let manager = RedisConnectionManager::new(config.redis_uri.expose_secret().as_str())
        .expect("redis uri should be valid");
    let cache = bb8::Pool::builder().build(manager).await.unwrap();

    let app = App::with(config).await;
    tracing::info!(
        host = app.host().to_string(),
        port = app.port(),
        "starting server"
    );
    app.serve(db, cache)
        .await
        .expect("The server should be running.");
}
