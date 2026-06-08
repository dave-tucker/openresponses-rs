use std::sync::Arc;

use axum::{Router, routing::get};
use clap::Parser;
use openresponses_server::MockHandler;

#[derive(Parser, Debug)]
#[command(name = "openresponses-server", about = "OpenResponses mock server")]
struct Cli {
    /// Port to listen on
    #[arg(long, env = "PORT", default_value = "3000")]
    port: u16,

    /// Optional API key for Bearer authentication
    #[arg(long, env = "API_KEY")]
    api_key: Option<String>,
}

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let handler = Arc::new(MockHandler::new());

    let app = Router::new()
        .nest("/v1", openresponses_axum::router(handler))
        .route("/health", get(health));

    let addr = format!("0.0.0.0:{}", cli.port);
    println!("openresponses-server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}
