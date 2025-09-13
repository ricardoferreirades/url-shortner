use crate::application::ShortenUrlUseCase;
use crate::domain::repositories::UrlRepository;

/// Application state that contains both use cases and repositories
#[derive(Clone)]
pub struct AppState<R>
where
    R: UrlRepository + Send + Sync + Clone,
{
    pub shorten_url_use_case: ShortenUrlUseCase<R>,
    pub url_repository: R,
}

impl<R> AppState<R>
where
    R: UrlRepository + Send + Sync + Clone,
{
    pub fn new(shorten_url_use_case: ShortenUrlUseCase<R>, url_repository: R) -> Self {
        Self {
            shorten_url_use_case,
            url_repository,
        }
    }
}
