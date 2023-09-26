use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// Composes layers into a full `tracing` subscriber.
///
/// `name` will be attached to all logged messages.
///
/// `default_level` is the default logging level to use if not set in the environment.
///  Should be one of "info", "warn", "debug", "error", or "trace".
///
/// `sink` is where all logs will be written. You can use this to optionally swallow logging.
///
/// # Implementation Notes
///
/// We're returning an impl rather than a concrete type for simplicity.
/// We also need `Send` and `Sync` to fully subscribe later on.
pub fn get_subscriber<Sink>(
    name: String,
    default_level: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    let format_layer = BunyanFormattingLayer::new(name, sink);
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
