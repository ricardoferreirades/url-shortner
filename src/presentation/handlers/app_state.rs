use crate::application::ShortenUrlUseCase;
use crate::domain::repositories::{
    AccountDeletionTokenRepository, PasswordResetRepository, UrlRepository, UserRepository,
};
use crate::domain::services::{AuthService, BulkProcessor, ProgressService, UrlService};
use crate::infrastructure::email::EmailSender;
use crate::infrastructure::PasswordResetRateLimiter;
use std::sync::Arc;

/// Application state that contains both use cases and repositories
#[derive(Clone)]
pub struct AppState<R, U, P, A>
where
    R: UrlRepository + Send + Sync + Clone + 'static,
    U: UserRepository + Send + Sync + Clone + 'static,
    P: PasswordResetRepository + Send + Sync + Clone,
    A: AccountDeletionTokenRepository + Send + Sync + Clone,
{
    pub shorten_url_use_case: ShortenUrlUseCase<R>,
    pub url_repository: R,
    pub url_service: UrlService<R>,
    pub auth_service: AuthService<U>,
    pub user_repository: U,
    pub progress_service: ProgressService,
    pub bulk_processor: BulkProcessor<R, U>,
    pub password_reset_repository: P,
    pub account_deletion_repository: A,
    pub email_sender: Option<Arc<dyn EmailSender>>,
    pub password_reset_rate_limiter: Arc<PasswordResetRateLimiter>,
}

impl<R, U, P, A> AppState<R, U, P, A>
where
    R: UrlRepository + Send + Sync + Clone + 'static,
    U: UserRepository + Send + Sync + Clone + 'static,
    P: PasswordResetRepository + Send + Sync + Clone,
    A: AccountDeletionTokenRepository + Send + Sync + Clone,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        shorten_url_use_case: ShortenUrlUseCase<R>,
        url_repository: R,
        url_service: UrlService<R>,
        auth_service: AuthService<U>,
        user_repository: U,
        password_reset_repository: P,
        account_deletion_repository: A,
        email_sender: Option<Arc<dyn EmailSender>>,
        password_reset_rate_limiter: Arc<PasswordResetRateLimiter>,
    ) -> Self {
        let progress_service = ProgressService::new();
        let bulk_processor = BulkProcessor::new(
            url_service.clone(),
            progress_service.clone(),
            user_repository.clone(),
        );

        Self {
            shorten_url_use_case,
            url_repository,
            url_service,
            auth_service,
            user_repository,
            progress_service,
            bulk_processor,
            password_reset_repository,
            account_deletion_repository,
            email_sender,
            password_reset_rate_limiter,
        }
    }
}
