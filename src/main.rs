use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use zero2prod::{app, config::get_configuration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = get_configuration().expect("Failed to read configuration.");

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database.connection_string())
        .await
        .context("Could not connect to database")?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.application_port))
        .await
        .expect("The socket should be available");

    app::serve(listener, db)
        .await
        .expect("The server should be running");

    Ok(())
}
