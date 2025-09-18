# Changelog

All notable changes to the URL Shortener project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Click entity with comprehensive domain model
- ClickRepository trait for analytics data access
- ClickTrackingService with async processing
- Database schema for clicks table with performance indexes
- Comprehensive test coverage for analytics features

### Changed
- Updated database schema to support analytics tracking
- Enhanced domain layer with analytics capabilities

## [0.1.0] - 2025-01-XX

### Added
- Core URL shortening functionality
- Database storage and retrieval with PostgreSQL + SQLx
- REST API endpoints with OpenAPI documentation
- Clean architecture with separation of concerns
- Comprehensive unit and integration tests
- Environment variable configuration
- Docker and development tools setup
- Health check endpoint (`/health`)

### Security
- Input validation and sanitization (OWASP compliance)
- Error handling improvements with custom error types
- Rate limiting and security headers (IP-based rate limiting, CSP, HSTS, X-Frame-Options)
- Malicious pattern detection (javascript:, data:, etc.)
- Scheme validation (http/https only)
- Length validation and input sanitization
- Structured JSON error responses

### Authentication & Authorization
- JWT-based authentication
- User registration/login
- User-specific URL management
- Custom short codes with validation (3-50 characters, alphanumeric + hyphens/underscores)
- Collision detection and error handling

### Infrastructure
- Multi-stage Docker builds
- Production Docker configuration
- Database migrations with performance indexes
- Analytics tables for click tracking
- Comprehensive middleware testing

### Technical Details

#### Database Schema Enhancement
- Added `user_id` column to urls table schema
- Updated database initialization script (init.sql)
- Verified fresh database deployment works correctly
- Fixed database dependency issues and schema mismatches
- Maintained backward compatibility with NULL user_id values
- Comprehensive testing of database schema changes

#### Rate Limiting & Security Implementation
- IP-based rate limiting with governor crate
- Configurable rate limits via environment variables
- Security headers middleware (CSP, HSTS, X-Frame-Options, etc.)
- Request body size limiting
- Request tracing and response compression
- Structured error responses for rate limit violations
- Comprehensive middleware testing
- OpenAPI documentation updates

#### Input Validation & Security
- URL format validation with configurable rules
- Malicious pattern detection (javascript:, data:, etc.)
- Scheme validation (http/https only)
- Length validation and input sanitization
- Custom error types with detailed messages
- Structured JSON error responses
- Short code validation
- Comprehensive test coverage (6 validation tests)

#### Custom Short Codes Implementation
- User-specified custom short codes via API
- Custom short code validation (3-50 characters, alphanumeric + hyphens/underscores)
- Collision detection and error handling
- Database schema updated to support longer codes (VARCHAR(50))
- Backward compatibility with auto-generated codes
- Comprehensive testing of custom code functionality

---

*For more information about upcoming features, see the [ROADMAP.md](ROADMAP.md)*
