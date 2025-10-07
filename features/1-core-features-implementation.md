# Core Features - Implementation Steps

## 1.1 URL Management
**Goal**: Complete URL lifecycle management with advanced features

### 1.1.1 URL Expiration & Management
- [x] Add expiration date field to URL entity
- [x] Implement expiration validation in URL service
- [x] Add scheduled cleanup for expired URLs
- [x] Create API endpoints for URL expiration management
- [x] Add expiration warnings and notifications

### 1.1.2 URL Deactivation
- [x] Add active/inactive status to URL entity
- [x] Implement soft delete functionality
- [x] Create deactivation API endpoints
- [x] Add status validation in redirect logic
- [x] Implement reactivation functionality

### 1.1.3 Bulk URL Management
- [x] Create bulk URL creation endpoint
- [x] Implement batch processing for URL operations
- [x] Add bulk status update functionality
- [x] Create bulk deletion with safety checks
- [x] Add progress tracking for bulk operations

## 1.2 User Management
**Goal**: Enhanced user account management and features

### 1.2.1 User Profiles
- [x] Extend user entity with profile fields
- [x] Create user profile management API
- [x] Add profile picture upload functionality
- [x] Implement profile validation and sanitization
- [x] Add profile privacy settings

### 1.2.2 Password Reset
- [x] Implement password reset token generation
- [x] Create secure password reset email system
- [ ] Add password reset API endpoints
- [ ] Implement token expiration and validation
- [ ] Add rate limiting for password reset requests

**Implementation Details:**
- Created PasswordResetToken entity with secure token generation
- Implemented PasswordResetService with UUID + random token generation
- Added PasswordResetRepository trait and PostgresPasswordResetRepository
- Added password_reset_tokens table with proper indexes
- Created EmailSender trait for email abstraction
- Implemented SmtpEmailSender with lettre library
- Added password reset email templates (text and HTML)
- Added SMTP configuration with environment variable support
- Implemented secure email sending with TLS/SSL

### 1.2.3 Account Deletion
- [ ] Implement secure account deletion process
- [ ] Add data anonymization for deleted accounts
- [ ] Create account deletion confirmation system
- [ ] Implement cascading deletion for user data
- [ ] Add account recovery grace period

## Implementation Strategy

### Phase 1: URL Lifecycle Management
- Start with URL expiration dates
- Add soft delete functionality
- Implement basic bulk operations

### Phase 2: User Account Enhancement
- Extend user profiles
- Add password reset functionality
- Implement account management features

### Phase 3: Advanced Features
- Complete bulk operations
- Add advanced user management
- Implement data retention policies

## Technical Considerations

### Database Changes
- [x] Add expiration_date column to urls table
- [x] Add status column for URL deactivation
- [x] Extend users table with profile fields
- [x] Add password_reset_tokens table

### API Design
- RESTful endpoints for all operations
- Proper HTTP status codes
- Input validation and error handling
- Rate limiting for sensitive operations

### Security
- Secure token generation for password reset
- Proper data sanitization
- Access control for user data
- Audit logging for sensitive operations
