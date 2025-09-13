// Clean Architecture Layers
mod domain;
mod application;
mod infrastructure;
mod presentation;

// Legacy modules (to be refactored)
mod database;
mod server;
mod shortener;
mod validation;
mod rate_limiting;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_server().await
}
