mod server;
mod shortener;
mod database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_server().await
}
