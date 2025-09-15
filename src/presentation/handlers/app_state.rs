use crate::application::ShortenUrlUseCase;
use crate::domain::repositories::{UrlRepository, UserRepository};
use crate::domain::services::AuthService;

/// Application state that contains both use cases and repositories
#[derive(Clone)]
pub struct AppState<R, U>
where
    R: UrlRepository + Send + Sync + Clone,
    U: UserRepository + Send + Sync + Clone,
{
    pub shorten_url_use_case: ShortenUrlUseCase<R>,
    pub url_repository: R,
    pub auth_service: AuthService<U>,
}

impl<R, U> AppState<R, U>
where
    R: UrlRepository + Send + Sync + Clone,
    U: UserRepository + Send + Sync + Clone,
{
    pub fn new(shorten_url_use_case: ShortenUrlUseCase<R>, url_repository: R, auth_service: AuthService<U>) -> Self {
        Self {
            shorten_url_use_case,
            url_repository,
            auth_service,
        }
    }
}
