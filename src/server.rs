use axum::{
    routing::get,
    Router,
    response::Html,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Create router with routes
    let app = Router::new()
        .route("/", get(welcome_handler))
        .layer(cors);
    
    // Create socket address
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8000));
    
    info!("Starting server on {}", addr);
    info!("Welcome to your app! Visit http://localhost:8000");
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn welcome_handler() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Welcome to Your App</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    max-width: 800px;
                    margin: 50px auto;
                    padding: 20px;
                    background-color: #f5f5f5;
                }
                .container {
                    background-color: white;
                    padding: 40px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    text-align: center;
                }
                h1 {
                    color: #333;
                    margin-bottom: 20px;
                }
                p {
                    color: #666;
                    font-size: 18px;
                    line-height: 1.6;
                }
                .highlight {
                    background-color: #e8f4fd;
                    padding: 20px;
                    border-radius: 5px;
                    border-left: 4px solid #2196F3;
                    margin: 20px 0;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🚀 Welcome to Your App!</h1>
                <p>Congratulations! Your Axum server is running successfully.</p>
                
                <div class="highlight">
                    <p><strong>Server Status:</strong> ✅ Running</p>
                    <p><strong>Port:</strong> 8000</p>
                    <p><strong>Framework:</strong> Axum (Rust)</p>
                </div>
                
                <p>You can now start building your URL shortener application!</p>
            </div>
        </body>
        </html>
    "#)
}
