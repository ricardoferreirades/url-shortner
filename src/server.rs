use axum::{
    routing::{get, post},
    Router,
    response::Html,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::shortener;
use crate::database::Database;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Get database URL from environment variable
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");
    
    // Connect to database
    let db = Database::new(&database_url).await?;
    info!("Connected to PostgreSQL database");
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // OpenAPI doc
    #[derive(OpenApi)]
    #[openapi(
        paths(
            shortener::shorten_url_handler,
            shortener::redirect_handler,
        ),
        components(
            schemas(
                shortener::ShortenUrlRequest,
                shortener::ShortenUrlResponse,
            )
        ),
        tags(
            (name = "url-shortener", description = "URL Shortener API")
        )
    )]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    // Create router with routes and docs
    let api_router = Router::new()
        .route("/", get(welcome_handler))
        .route("/shorten", post(shortener::shorten_url_handler))
        .route("/:short_code", get(shortener::redirect_handler));

    let app = api_router
        .merge(
            SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi)
        )
        .with_state(db)
        .layer(cors);
    
    // Get server configuration from environment variables
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    // Create socket address
    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse()?;
    
    info!("Starting server on {}", addr);
    info!("Welcome to your app! Visit http://{}:{}", host, port);
    info!("URL shortening endpoint: POST http://{}:{}/shorten", host, port);
    info!("Redirect endpoint: GET http://{}:{}/{{short_code}}", host, port);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

pub async fn welcome_handler() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>URL Shortener</title>
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
                    background-color: #e8f8fd;
                    padding: 20px;
                    border-radius: 5px;
                    border-left: 4px solid #2196F3;
                    margin: 20px 0;
                }
                .endpoint {
                    background-color: #f0f8ff;
                    padding: 15px;
                    border-radius: 5px;
                    border: 1px solid #ddd;
                    font-family: monospace;
                    margin: 10px 0;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🔗 URL Shortener</h1>
                <p>Welcome to your URL shortener application!</p>
                
                <div class="highlight">
                    <p><strong>Server Status:</strong> ✅ Running</p>
                    <p><strong>Port:</strong> 8000</p>
                    <p><strong>Framework:</strong> Axum (Rust)</p>
                </div>
                
                <h3>API Endpoints:</h3>
                <div class="endpoint">
                    <strong>POST /shorten</strong><br>
                    Send JSON: {"url": "https://example.com/very/long/url"}<br>
                    Returns: {"short_url": "http://localhost:8000/abc123", "original_url": "..."}
                </div>
                <div class="endpoint">
                    <strong>GET /{short_code}</strong><br>
                    Redirects to the original URL<br>
                    Example: http://localhost:8000/abc123
                </div>
                
                <p>Test it with curl or any HTTP client!</p>
            </div>
        </body>
        </html>
    "#)
}

