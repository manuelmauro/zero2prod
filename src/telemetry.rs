use tokio::task::JoinHandle;
use tracing::Subscriber;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt};

pub fn get_subscriber<W>(env_filter: &str, writer: W) -> impl Subscriber + Send + Sync
where
    W: for<'writer> MakeWriter<'writer> + 'static + Send + Sync,
{
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                env_filter.into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(writer))
}

/// Spawns a blocking task with tracing support.
pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
