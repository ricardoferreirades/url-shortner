// Clean Architecture Layers
mod domain;
mod application;
mod infrastructure;
mod presentation;

// All modules are now organized within clean architecture layers

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    infrastructure::server::start_server().await
}
