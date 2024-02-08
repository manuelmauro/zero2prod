use crate::email::EmailClient;
use axum::{http::Request, Router};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod health;
mod subscription;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    email_client: EmailClient,
}

fn app_router() -> Router<AppState> {
    health::router().merge(subscription::router())
}

pub async fn serve(
    listener: TcpListener,
    db: PgPool,
    email_client: EmailClient,
) -> Result<(), std::io::Error> {
    let app = app_router()
        .with_state(AppState { db, email_client })
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let id = uuid::Uuid::new_v4();
                tracing::info_span!(
                    "request",
                    method = ?request.method(),
                    uri = ?request.uri(),
                    %id,
                )
            }),
        );

    axum::serve(listener, app.into_make_service()).await
}
