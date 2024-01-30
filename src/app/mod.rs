use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;

mod health;
mod subscription;

pub async fn serve(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health::health_check))
        .route("/subscribe", post(subscription::subscribe));

    axum::serve(listener, app).await
}
