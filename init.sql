-- Create the urls table
CREATE TABLE IF NOT EXISTS urls (
    id SERIAL PRIMARY KEY,
    short_code VARCHAR(6) UNIQUE NOT NULL,
    original_url TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    user_id INTEGER
);

-- Create an index on short_code for faster lookups
CREATE INDEX IF NOT EXISTS idx_urls_short_code ON urls(short_code);
