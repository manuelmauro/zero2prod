use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::{app, config::get_configuration, telemetry::get_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    get_subscriber("zero2prod=trace,tower_http=trace,axum::rejection=trace").init();

    let config = get_configuration().expect("Failed to read configuration.");

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database.connection_string())
        .await
        .context("Could not connect to database")?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.application_port))
        .await
        .expect("The socket should be available");

    tracing::info!(port = config.application_port, "starting server");
    app::serve(listener, db)
        .await
        .expect("The server should be running");

    Ok(())
}
