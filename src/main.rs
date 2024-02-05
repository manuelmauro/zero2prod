use anyhow::Context;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::{app, config::get_configuration, telemetry::get_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_configuration().expect("Failed to read configuration.");

    get_subscriber(&config.application.log_level, std::io::stderr).init();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect_lazy(config.database.connection_string().expose_secret())
        .context("Could not connect database connection pool")?;

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))
    .await
    .expect("The socket should be available");

    tracing::info!(port = config.application.port, "starting server");
    app::serve(listener, db)
        .await
        .expect("The server should be running");

    Ok(())
}
