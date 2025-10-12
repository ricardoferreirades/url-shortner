// Clean Architecture Layers
mod application;
mod domain;
mod infrastructure;
mod presentation;

// All modules are now organized within clean architecture layers

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    infrastructure::server::start_server().await
}
