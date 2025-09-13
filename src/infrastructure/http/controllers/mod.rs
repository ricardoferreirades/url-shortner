// HTTP Controllers for different frameworks
// This allows us to have framework-specific implementations

pub mod axum_controller;

// Future: pub mod actix_controller;
// Future: pub mod warp_controller;

pub use axum_controller::*;
