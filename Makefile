.PHONY: up down logs db-shell test fmt lint check

# Start the database
up:
	docker-compose up -d

# Stop the database
down:
	docker-compose down

# View database logs
logs:
	docker-compose logs -f postgres

# Connect to database shell
db-shell:
	docker-compose exec postgres psql -U postgres -d url_shortener

# Run the application
run:
	cargo run

# Test the application
test:
	curl -X POST http://localhost:8000/shorten -H "Content-Type: application/json" -d '{"url": "https://example.com/test"}'

# Format code (like Prettier)
fmt:
	cargo fmt

# Lint code (like ESLint)
lint:
	cargo clippy

# Check code without building (faster)
check:
	cargo check

# Check spelling
spell:
	typos

# Fix spelling issues automatically
spell-fix:
	typos --write-changes

# Run all code quality checks
quality: fmt lint check spell
