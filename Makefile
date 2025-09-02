.PHONY: up down logs db-shell test

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
