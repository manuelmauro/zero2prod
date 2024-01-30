use axum::Router;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod health;
mod subscription;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
}

fn app_router() -> Router<AppState> {
    health::router().merge(subscription::router())
}

pub async fn serve(listener: TcpListener, db: PgPool) -> Result<(), std::io::Error> {
    let app = app_router()
        .with_state(AppState { db })
        .layer(TraceLayer::new_for_http());

    axum::serve(listener, app.into_make_service()).await
}
