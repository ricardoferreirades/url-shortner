use axum::{
    middleware,
    response::Html,
    routing::{get, post, put, delete, patch},
    Json, Router,
};
use serde_json::json;
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Clean Architecture imports
use crate::domain::UrlService;
use crate::application::{ShortenUrlUseCase, ShortenUrlRequest};
use crate::application::dto::requests::BulkShortenUrlsRequest;
use crate::infrastructure::{PostgresUrlRepository, PostgresUserRepository};
use crate::domain::services::AuthService;
use crate::presentation::{
    shorten_url_handler, redirect_handler, register_handler, login_handler, AppState,
    bulk_shorten_urls_handler, batch_url_operations_handler, bulk_status_update_handler,
    bulk_expiration_update_handler, bulk_delete_handler,
    deactivate_url_handler, reactivate_url_handler,
    get_expiration_info_handler, set_expiration_handler, extend_expiration_handler, get_expiring_urls_handler,
    async_bulk_shorten_urls_handler, async_batch_url_operations_handler,
    get_bulk_operation_progress_handler, cancel_bulk_operation_handler, get_user_operations_handler
};
use crate::presentation::handlers::url_handlers::{__path_shorten_url_handler, __path_redirect_handler, __path_deactivate_url_handler, __path_reactivate_url_handler, __path_bulk_shorten_urls_handler};
use crate::presentation::handlers::auth_handlers::{__path_register_handler, __path_login_handler};
use crate::presentation::handlers::expiration_handlers::{__path_get_expiration_info_handler, __path_set_expiration_handler, __path_extend_expiration_handler, __path_get_expiring_urls_handler};
use crate::infrastructure::rate_limiting::{
    create_request_size_layer, create_tracing_layer_simple, create_compression_layer_simple,
    security_headers_middleware, rate_limit_middleware, RateLimitConfig,
};

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get database URL: prefer DATABASE_URL; otherwise, assemble from POSTGRES_* parts (shared with Docker)
    let database_url = if let Ok(url) = env::var("DATABASE_URL") {
        url
    } else {
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
        let name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set (or provide DATABASE_URL)");
        let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set (or provide DATABASE_URL)");
        let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set (or provide DATABASE_URL)");
        format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, name)
    };

    // Connect to database using new clean architecture
    let pool = sqlx::PgPool::connect(&database_url).await?;
    let url_repository = PostgresUrlRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool);
    info!("Connected to PostgreSQL database with clean architecture");

    // Configure rate limiting
    let rate_limit_config = RateLimitConfig {
        requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60),
        burst_size: env::var("RATE_LIMIT_BURST_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10),
        max_request_size: env::var("MAX_REQUEST_SIZE")
            .unwrap_or_else(|_| "1048576".to_string()) // 1MB
            .parse()
            .unwrap_or(1024 * 1024),
    };

    info!(
        "Rate limiting configured: {} req/min, burst: {}, max size: {} bytes",
        rate_limit_config.requests_per_minute,
        rate_limit_config.burst_size,
        rate_limit_config.max_request_size
    );

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create clean architecture components
    let url_service = UrlService::new(url_repository.clone());
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let shorten_url_use_case = ShortenUrlUseCase::new(url_service.clone(), base_url);
    
    // Create auth service
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let auth_service = AuthService::new(user_repository.clone(), jwt_secret);
    
    // Create application state
    let app_state = AppState::new(shorten_url_use_case, url_repository, url_service, auth_service, user_repository);

    // OpenAPI doc
    #[derive(OpenApi)]
    #[openapi(
        paths(
            register_handler,
            login_handler,
            shorten_url_handler,
            bulk_shorten_urls_handler,
            batch_url_operations_handler,
            bulk_status_update_handler,
            bulk_expiration_update_handler,
            bulk_delete_handler,
            async_bulk_shorten_urls_handler,
            async_batch_url_operations_handler,
            get_bulk_operation_progress_handler,
            cancel_bulk_operation_handler,
            get_user_operations_handler,
            redirect_handler,
            deactivate_url_handler,
            reactivate_url_handler,
            get_expiration_info_handler,
            set_expiration_handler,
            extend_expiration_handler,
            get_expiring_urls_handler,
            health_check,
        ),
        components(
            schemas(
                ShortenUrlRequest,
                BulkShortenUrlsRequest,
                crate::application::ShortenUrlResponse,
                crate::application::ErrorResponse,
                crate::infrastructure::rate_limiting::RateLimitError,
                crate::presentation::handlers::auth_handlers::RegisterRequest,
                crate::presentation::handlers::auth_handlers::LoginRequest,
                crate::presentation::handlers::auth_handlers::AuthResponse,
                crate::presentation::handlers::auth_handlers::UserResponse,
            )
        ),
        tags(
            (name = "url-shortener", description = "URL Shortener API")
        )
    )]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    // Create router with routes and docs using clean architecture
    let api_router = Router::new()
        .route("/", get(welcome_handler))
        .route("/health", get(health_check))
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/shorten", post(shorten_url_handler))
        .route("/:short_code", get(redirect_handler))
        // Bulk operations (synchronous)
        .route("/urls/bulk", post(bulk_shorten_urls_handler))
        .route("/urls/batch", post(batch_url_operations_handler))
        .route("/urls/bulk/status", patch(bulk_status_update_handler))
        .route("/urls/bulk/expiration", patch(bulk_expiration_update_handler))
        .route("/urls/bulk", delete(bulk_delete_handler))
        // Async bulk operations with progress tracking
        .route("/urls/bulk/async", post(async_bulk_shorten_urls_handler))
        .route("/urls/batch/async", post(async_batch_url_operations_handler))
        // Progress tracking endpoints
        .route("/urls/bulk/progress/:operation_id", get(get_bulk_operation_progress_handler))
        .route("/urls/bulk/progress/:operation_id", delete(cancel_bulk_operation_handler))
        .route("/urls/bulk/operations", get(get_user_operations_handler))
        // URL management endpoints
        .route("/urls/:id", delete(deactivate_url_handler))
        .route("/urls/:id/reactivate", patch(reactivate_url_handler))
        // Expiration management endpoints
        .route("/urls/:short_code/expiration", get(get_expiration_info_handler))
        .route("/urls/:short_code/expiration", put(set_expiration_handler))
        .route("/urls/:short_code/extend", post(extend_expiration_handler))
        .route("/urls/expiring-soon", get(get_expiring_urls_handler));

    let app = api_router
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi))
        .with_state(app_state)
        .layer(cors)
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(create_request_size_layer(&rate_limit_config))
        .layer(create_tracing_layer_simple())
        .layer(create_compression_layer_simple());

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
    info!("Health check endpoint: GET http://{}:{}/health", host, port);
    info!(
        "URL shortening endpoint: POST http://{}:{}/shorten",
        host, port
    );
    info!(
        "Redirect endpoint: GET http://{}:{}/{{short_code}}",
        host, port
    );
    info!("API documentation: http://{}:{}/docs", host, port);
    info!("Security features enabled: rate limiting, security headers, compression");

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn welcome_handler() -> Html<&'static str> {
    Html(
        r#"
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
    "#,
    )
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check successful", body = serde_json::Value)
    )
)]
pub async fn health_check() -> Json<serde_json::Value> {
    let health_status = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "url-shortener",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": "running"
    });

    info!("Health check requested - service is healthy");
    Json(health_status)
}
