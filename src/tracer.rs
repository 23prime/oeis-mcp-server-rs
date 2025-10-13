use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

const DEFAULT_LEVEL: &str = "debug";

pub fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| DEFAULT_LEVEL.to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
