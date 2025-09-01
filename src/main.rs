mod server;
mod shortener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_server().await
}
