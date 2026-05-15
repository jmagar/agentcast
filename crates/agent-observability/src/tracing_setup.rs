use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(default_filter: &str) -> Result<(), String> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init()
        .map_err(|error| error.to_string())
}
