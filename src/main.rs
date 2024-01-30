use zero2prod::{app, config::get_configuration};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let _configuration = get_configuration().expect("Failed to read configuration.");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("The socket should be available");

    app::serve(listener)
        .await
        .expect("The server should be running");
}
