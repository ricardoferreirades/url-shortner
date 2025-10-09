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
- Third generic parameter (P) to AppState for PasswordResetRepository integration
- Batch URL operations: deactivate, reactivate, delete, update_status, update_expiration
- ConcreteAppState type alias for non-generic handlers
- Dead code allowances for future-feature methods and structs across all layers

### Changed
- **BREAKING**: URL handlers refactored from single 4,962-line file into modular structure
  - Created `url_handlers/urls/` directory with 11 focused handler files
  - Each handler now in separate file (~70-125 lines)
  - Improved code organization and maintainability
- AppState now requires 3 generic parameters: `<R, U, P>` instead of `<R, U>`
- Updated all handler signatures to include PasswordResetRepository type parameter
- Enhanced error handling in password reset service (replaced RepositoryError boxing with Internal)
- Improved borrow checker compliance in file upload and profile handlers
- Updated database schema to support analytics tracking
- Enhanced domain layer with analytics capabilities

### Fixed
- 136+ compilation errors resolved across the codebase
- 148 compiler warnings eliminated (100% reduction)
- Removed 50 duplicate handler functions (were identical copies with different names)
- Fixed Multipart import (changed from `axum` to `axum_extra`)
- Fixed BatchOperationResult import paths
- Resolved StdError dynamic trait object size issues (7 instances)
- Fixed borrow/ownership issues in user.privacy and file_data
- Changed `img.dimensions()` to `(img.width(), img.height())` for image crate compatibility
- Removed unused mut variables (batch_size, token, email_builder)
- Fixed error conversion issues with `?` operator in password reset flow

### Removed
- Deleted url_handlers.rs (replaced with modular structure)
- Removed 50 duplicate URL handler files
- Cleaned up 40+ unused imports across handlers and services
- Removed unnecessary __path_* imports from server.rs

### Security
- Added 'static lifetime bounds to AppState generic parameters for thread safety
- Enhanced error handling with explicit match statements instead of map_err chains

### Infrastructure  
- Upgraded sqlx from v0.7.4 to v0.8.6 for Rust Edition 2024 compatibility
- Resolved future incompatibility warnings (never type fallback)
- All handlers now properly integrated with dependency injection

### Technical Debt
- Temporarily disabled OpenAPI path declarations (TODO: re-enable after proper re-exports)
- Added comprehensive #[allow(dead_code)] attributes to future-use code
- Documented future work needed for full OpenAPI documentation restoration

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
