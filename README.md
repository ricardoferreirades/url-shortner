# URL Shortener

This is a simple URL shortener written in Rust.

## Environment Setup

1. Copy the example environment file:
   ```bash
   cp .env-example .env
   ```

2. Edit the `.env` file with your actual database credentials:
   ```
   DATABASE_URL=postgresql://username:password@localhost:5432/database_name
   HOST=127.0.0.1
   PORT=8000
   ```

3. Make sure your PostgreSQL database is running and accessible with the credentials in your `.env` file.

## Running the Application

```bash
cargo run
```

The server will start on the host and port specified in your `.env` file (default: http://127.0.0.1:8000).

## Technologies Used

- **Rust** - Systems programming language for performance and safety
- **Axum** - Modern web framework for Rust with async support
- **Tokio** - Asynchronous runtime for Rust
- **SQLx** - Async SQL toolkit with compile-time checked queries
- **PostgreSQL** - Robust relational database system
- **Serde** - Serialization/deserialization framework for Rust
- **Tower HTTP** - HTTP middleware and utilities
- **dotenv** - Environment variable management
- **Tracing** - Structured logging and diagnostics
- **Docker** - Containerization platform for consistent development environments
- **Make** - Build automation tool for common development tasks