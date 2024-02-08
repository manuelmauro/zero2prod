use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::{app, config::get_configuration, email::EmailClient, telemetry::get_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_configuration().expect("The configuration should load.");

    get_subscriber(&config.application.log_level, std::io::stderr).init();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect_lazy_with(config.database.with_db());

    let email_client = EmailClient::new(
        config.email_client.base_url,
        config
            .email_client
            .sender_email
            .try_into()
            .expect("The sender email should be valid."),
    );

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))
    .await
    .expect("The listener should be able to bind the address.");

    tracing::info!(
        host = config.application.host,
        port = config.application.port,
        "starting server"
    );
    app::serve(listener, db, email_client)
        .await
        .expect("The server should be running.");

    Ok(())
}
