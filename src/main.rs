#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    zero2prod::serve().await.expect("Server should be running");
}
