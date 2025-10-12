pub mod app_state;
pub mod auth_handlers;
pub mod expiration_handlers;
pub mod file_upload_handlers;
pub mod password_reset_handlers;
pub mod privacy_handlers;
pub mod profile_handlers;
pub mod progress_handlers;
pub mod url_handlers;

pub use app_state::*;
pub use auth_handlers::*;
pub use expiration_handlers::*;
pub use file_upload_handlers::*;
pub use password_reset_handlers::*;
pub use privacy_handlers::*;
pub use profile_handlers::*;
pub use progress_handlers::*;
pub use url_handlers::*;

// Type alias for the concrete AppState used in the application
pub type ConcreteAppState = app_state::AppState<
    crate::infrastructure::database::PostgresUrlRepository,
    crate::infrastructure::database::PostgresUserRepository,
    crate::infrastructure::database::PostgresPasswordResetRepository,
>;
