use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

mod counter;
mod oeis;
mod oeis_client;

use counter::Counter;
use oeis::OEIS;

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Streamable HTTP server...");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // TODO: remove
    let counter_service = StreamableHttpService::new(
        || Ok(Counter::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let service = StreamableHttpService::new(
        || Ok(OEIS::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new()
        .nest_service("/mcp/counter", counter_service)
        .nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;

    let server = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() });

    println!("🚀 Streamable HTTP server is ready!");

    let _ = server.await;
    Ok(())
}
