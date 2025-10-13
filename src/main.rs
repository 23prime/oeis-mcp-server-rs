use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};

mod oeis;
mod oeis_client;
mod tracer;

use oeis::OEIS;
use tracer::setup_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔄 Starting OEIS MCP server...");
    setup_tracing();

    let port = get_port_from_env();
    let bind_address = format!("127.0.0.1:{}", port);

    let service = StreamableHttpService::new(
        || Ok(OEIS::new()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(&bind_address).await?;

    let server = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() });

    println!("🚀 OEIS MCP server is ready at {}", &bind_address);

    let _ = server.await;
    Ok(())
}

fn get_port_from_env() -> String {
    std::env::var("PORT").unwrap_or_else(|_| "8000".to_string())
}
