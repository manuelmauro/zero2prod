use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

pub async fn serve() -> Result<(), std::io::Error> {
    let app = Router::new().route("/health_check", get(health_check));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, app).await
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
