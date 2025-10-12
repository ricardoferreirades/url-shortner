use crate::application::dto::requests::{BatchOperationData, BatchOperationType};
use crate::domain::repositories::{UrlRepository, UserRepository};
use crate::domain::services::{ProgressService, UrlService};
use std::sync::Arc;
use tokio::task;
use tracing::{error, info};

/// Service for processing bulk operations in the background
#[derive(Clone)]
pub struct BulkProcessor<R, U>
where
    R: UrlRepository + Send + Sync + Clone + 'static,
    U: UserRepository + Send + Sync + Clone + 'static,
{
    url_service: UrlService<R>,
    progress_service: ProgressService,
    _user_repository: Arc<U>,
}

impl<R, U> BulkProcessor<R, U>
where
    R: UrlRepository + Send + Sync + Clone + 'static,
    U: UserRepository + Send + Sync + Clone + 'static,
{
    pub fn new(
        url_service: UrlService<R>,
        progress_service: ProgressService,
        user_repository: U,
    ) -> Self {
        Self {
            url_service,
            progress_service,
            _user_repository: Arc::new(user_repository),
        }
    }

    /// Process a bulk operation in the background
    pub async fn process_bulk_operation(
        &self,
        operation_id: String,
        operation: BatchOperationType,
        url_ids: Vec<i32>,
        data: Option<BatchOperationData>,
        user_id: Option<i32>,
    ) -> Result<(), BulkProcessorError> {
        let total_items = url_ids.len();

        // Update status to processing
        if let Err(e) = self
            .progress_service
            .update_status(
                &operation_id,
                crate::application::dto::responses::BulkOperationStatus::Processing,
            )
            .await
        {
            error!("Failed to update operation status to processing: {}", e);
            return Err(BulkProcessorError::ProgressUpdateFailed(e.to_string()));
        }

        info!(
            "Starting bulk operation {} for {} URLs",
            operation_id, total_items
        );

        // Spawn background task
        let url_service = self.url_service.clone();
        let progress_service = self.progress_service.clone();

        task::spawn(async move {
            let mut processed_items = 0;
            let mut successful_items = 0;
            let mut failed_items = 0;
            let batch_size = 10; // Process in batches of 10

            // Process URLs in batches
            for chunk in url_ids.chunks(batch_size) {
                // Check if operation was cancelled
                if let Ok(progress) = progress_service.get_progress(&operation_id).await {
                    if matches!(
                        progress.status,
                        crate::application::dto::responses::BulkOperationStatus::Cancelled
                    ) {
                        info!(
                            "Operation {} was cancelled, stopping processing",
                            operation_id
                        );
                        break;
                    }
                }

                // Process current batch
                let batch_result = match &operation {
                    BatchOperationType::Deactivate => {
                        url_service.batch_deactivate_urls(chunk, user_id).await
                    }
                    BatchOperationType::Reactivate => {
                        url_service.batch_reactivate_urls(chunk, user_id).await
                    }
                    BatchOperationType::Delete => {
                        url_service.batch_delete_urls(chunk, user_id).await
                    }
                    BatchOperationType::UpdateStatus => {
                        if let Some(ref batch_data) = data {
                            if let Some(ref status_str) = batch_data.status {
                                let status = match status_str.as_str() {
                                    "active" => crate::domain::entities::UrlStatus::Active,
                                    "inactive" => crate::domain::entities::UrlStatus::Inactive,
                                    _ => {
                                        error!("Invalid status in batch operation: {}", status_str);
                                        continue;
                                    }
                                };
                                url_service
                                    .batch_update_status(chunk, status, user_id)
                                    .await
                            } else {
                                error!("No status provided for UpdateStatus operation");
                                continue;
                            }
                        } else {
                            error!("No data provided for UpdateStatus operation");
                            continue;
                        }
                    }
                    BatchOperationType::UpdateExpiration => {
                        let expiration_date = data.as_ref().and_then(|d| d.expiration_date);
                        url_service
                            .batch_update_expiration(chunk, expiration_date, user_id)
                            .await
                    }
                };

                match batch_result {
                    Ok(result) => {
                        processed_items += result.total_processed;
                        successful_items += result.successful;
                        failed_items += result.failed;

                        // Update progress
                        if let Err(e) = progress_service
                            .update_progress(
                                &operation_id,
                                processed_items,
                                successful_items,
                                failed_items,
                            )
                            .await
                        {
                            error!(
                                "Failed to update progress for operation {}: {}",
                                operation_id, e
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            "Batch operation failed for operation {}: {}",
                            operation_id, e
                        );
                        failed_items += chunk.len();
                        processed_items += chunk.len();

                        // Update progress even on failure
                        if let Err(progress_err) = progress_service
                            .update_progress(
                                &operation_id,
                                processed_items,
                                successful_items,
                                failed_items,
                            )
                            .await
                        {
                            error!(
                                "Failed to update progress after batch failure: {}",
                                progress_err
                            );
                        }
                    }
                }

                // Small delay between batches to prevent overwhelming the system
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Final status update
            let final_status = if processed_items >= total_items {
                if failed_items == 0 {
                    crate::application::dto::responses::BulkOperationStatus::Completed
                } else if successful_items == 0 {
                    crate::application::dto::responses::BulkOperationStatus::Failed
                } else {
                    crate::application::dto::responses::BulkOperationStatus::Completed
                }
            } else {
                crate::application::dto::responses::BulkOperationStatus::Failed
            };

            if let Err(e) = progress_service
                .update_status(&operation_id, final_status)
                .await
            {
                error!(
                    "Failed to update final status for operation {}: {}",
                    operation_id, e
                );
            }

            info!(
                "Completed bulk operation {}: {}/{} successful, {}/{} failed",
                operation_id, successful_items, total_items, failed_items, total_items
            );
        });

        Ok(())
    }

    /// Process bulk URL creation in the background
    pub async fn process_bulk_url_creation(
        &self,
        operation_id: String,
        urls: Vec<crate::application::dto::requests::ShortenUrlRequest>,
        user_id: Option<i32>,
    ) -> Result<(), BulkProcessorError> {
        let total_items = urls.len();

        // Update status to processing
        if let Err(e) = self
            .progress_service
            .update_status(
                &operation_id,
                crate::application::dto::responses::BulkOperationStatus::Processing,
            )
            .await
        {
            error!("Failed to update operation status to processing: {}", e);
            return Err(BulkProcessorError::ProgressUpdateFailed(e.to_string()));
        }

        info!(
            "Starting bulk URL creation {} for {} URLs",
            operation_id, total_items
        );

        // Spawn background task
        let url_service = self.url_service.clone();
        let progress_service = self.progress_service.clone();

        task::spawn(async move {
            let mut processed_items = 0;
            let mut successful_items = 0;
            let mut failed_items = 0;

            for url_request in urls {
                // Check if operation was cancelled
                if let Ok(progress) = progress_service.get_progress(&operation_id).await {
                    if matches!(
                        progress.status,
                        crate::application::dto::responses::BulkOperationStatus::Cancelled
                    ) {
                        info!(
                            "Operation {} was cancelled, stopping processing",
                            operation_id
                        );
                        break;
                    }
                }

                // Process individual URL creation
                let custom_short_code = url_request
                    .custom_short_code
                    .and_then(|code| crate::domain::entities::ShortCode::new(code).ok());

                match url_service
                    .create_url(
                        &url_request.url,
                        custom_short_code,
                        url_request.expiration_date,
                        user_id,
                    )
                    .await
                {
                    Ok(_) => {
                        successful_items += 1;
                    }
                    Err(e) => {
                        error!(
                            "Failed to create URL in bulk operation {}: {}",
                            operation_id, e
                        );
                        failed_items += 1;
                    }
                }

                processed_items += 1;

                // Update progress
                if let Err(e) = progress_service
                    .update_progress(
                        &operation_id,
                        processed_items,
                        successful_items,
                        failed_items,
                    )
                    .await
                {
                    error!(
                        "Failed to update progress for operation {}: {}",
                        operation_id, e
                    );
                }

                // Small delay between operations
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            // Final status update
            let final_status = if processed_items >= total_items {
                if failed_items == 0 {
                    crate::application::dto::responses::BulkOperationStatus::Completed
                } else if successful_items == 0 {
                    crate::application::dto::responses::BulkOperationStatus::Failed
                } else {
                    crate::application::dto::responses::BulkOperationStatus::Completed
                }
            } else {
                crate::application::dto::responses::BulkOperationStatus::Failed
            };

            if let Err(e) = progress_service
                .update_status(&operation_id, final_status)
                .await
            {
                error!(
                    "Failed to update final status for operation {}: {}",
                    operation_id, e
                );
            }

            info!(
                "Completed bulk URL creation {}: {}/{} successful, {}/{} failed",
                operation_id, successful_items, total_items, failed_items, total_items
            );
        });

        Ok(())
    }
}

/// Bulk processor errors
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum BulkProcessorError {
    #[error("Progress update failed: {0}")]
    ProgressUpdateFailed(String),

    #[error("Operation processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Invalid operation data: {0}")]
    InvalidData(String),
}
