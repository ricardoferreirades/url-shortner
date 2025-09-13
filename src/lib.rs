// Clean Architecture Layers
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

// Legacy modules (to be refactored)
pub mod database;
pub mod server;
pub mod shortener;
pub mod validation;
pub mod rate_limiting;
