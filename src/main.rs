use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::util::SubscriberInitExt;
use zero2prod::{app::App, config::get_configuration, telemetry::get_subscriber};

#[tokio::main]
async fn main() {
    let config = get_configuration().expect("The configuration should load.");
    get_subscriber(&config.application.log_level, std::io::stderr).init();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect_lazy_with(config.database.with_db());
    let app = App::with(config).await;

    tracing::info!(
        host = app.host().to_string(),
        port = app.port(),
        "starting server"
    );
    app.serve(db).await.expect("The server should be running.");
}
