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

## Development Commands

This project uses `make` for common development tasks. Here are the available commands:

### Database Management

```bash
make up          # Start PostgreSQL database (Docker)
make down        # Stop PostgreSQL database
make logs        # View database logs
make db-shell    # Connect to database shell
```

### Running the Application

```bash
make run         # Run the application (same as cargo run)
cargo run        # Alternative: run directly with cargo
```

The server will start on the host and port specified in your `.env` file (default: http://127.0.0.1:8000).

```bash
make test        # Test the application with a sample request
```

### Code Quality

```bash
make fmt         # Format code (cargo fmt + taplo for TOML files)
make lint        # Lint code with clippy
make check       # Type-check code without building (faster)
make spell       # Check spelling with typos-cli
make spell-fix   # Automatically fix spelling issues
make quality     # Run all quality checks (fmt + lint + check + spell)
```

### Git Hooks

```bash
make setup-hooks # Install git hooks (pre-push quality checks)
```

The pre-push hook automatically runs:
- Code formatting check
- Linting (clippy with warnings as errors)
- Spell checking
- Type checking
- All tests

After running all checks, a summary table displays the status of each check, making it easy to see what passed or failed at a glance.

## Technologies Used

### Core Technologies
- **Rust** - Systems programming language for performance and safety
- **Axum** - Modern web framework for Rust with async support
- **Tokio** - Asynchronous runtime for Rust
- **SQLx** - Async SQL toolkit with compile-time checked queries
- **PostgreSQL** - Robust relational database system
- **Serde** - Serialization/deserialization framework for Rust
- **Tower HTTP** - HTTP middleware and utilities
- **dotenv** - Environment variable management
- **Tracing** - Structured logging and diagnostics

### Development Tools
- **Docker** - Containerization platform for consistent development environments
- **Make** - Build automation tool for common development tasks
- **Clippy** - Rust linter for catching common mistakes and improving code
- **rustfmt** - Code formatter for consistent Rust style
- **taplo** - TOML formatter and linter
- **typos-cli** - Fast spell checker for source code
- **Git Hooks** - Automated pre-push quality checks