use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;

pub fn get_subscriber(env_filter: &str) -> impl Subscriber + Send + Sync {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                env_filter.into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
}
