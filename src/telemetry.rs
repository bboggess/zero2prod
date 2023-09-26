use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// Composes layers into a full `tracing` subscriber.
///
/// # Implementation Notes
///
/// We're returning an impl rather than a concrete type for simplicity.
/// We also need `Send` and `Sync` to fully subscribe later on.
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let format_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(format_layer)
}

/// Registers a global default subscriber for our telemetry.
///
/// Do not call this multiple times!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("Failed to subscribe to tracing.");
}
