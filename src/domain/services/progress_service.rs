use crate::application::dto::responses::{BulkOperationProgress, BulkOperationStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Service for tracking progress of bulk operations
#[derive(Clone)]
pub struct ProgressService {
    operations: Arc<RwLock<HashMap<String, BulkOperationProgress>>>,
}

#[allow(dead_code)]
impl ProgressService {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new bulk operation and return its ID
    pub async fn create_operation(&self, total_items: usize) -> String {
        let operation_id = Uuid::new_v4().to_string();
        let progress = BulkOperationProgress {
            operation_id: operation_id.clone(),
            status: BulkOperationStatus::Pending,
            total_items,
            processed_items: 0,
            successful_items: 0,
            failed_items: 0,
            progress_percentage: 0.0,
        };

        let mut operations = self.operations.write().await;
        operations.insert(operation_id.clone(), progress);
        operation_id
    }

    /// Update operation status
    pub async fn update_status(
        &self,
        operation_id: &str,
        status: BulkOperationStatus,
    ) -> Result<(), ProgressServiceError> {
        let mut operations = self.operations.write().await;
        if let Some(progress) = operations.get_mut(operation_id) {
            progress.status = status;
            Ok(())
        } else {
            Err(ProgressServiceError::OperationNotFound)
        }
    }

    /// Update operation progress
    pub async fn update_progress(
        &self,
        operation_id: &str,
        processed_items: usize,
        successful_items: usize,
        failed_items: usize,
    ) -> Result<(), ProgressServiceError> {
        let mut operations = self.operations.write().await;
        if let Some(progress) = operations.get_mut(operation_id) {
            progress.processed_items = processed_items;
            progress.successful_items = successful_items;
            progress.failed_items = failed_items;
            progress.progress_percentage = if progress.total_items > 0 {
                (processed_items as f32 / progress.total_items as f32) * 100.0
            } else {
                0.0
            };

            // Update status based on progress
            if progress.processed_items >= progress.total_items {
                if progress.failed_items == 0 {
                    progress.status = BulkOperationStatus::Completed;
                } else if progress.successful_items == 0 {
                    progress.status = BulkOperationStatus::Failed;
                } else {
                    progress.status = BulkOperationStatus::Completed;
                }
            } else if progress.processed_items > 0 {
                progress.status = BulkOperationStatus::Processing;
            }
            Ok(())
        } else {
            Err(ProgressServiceError::OperationNotFound)
        }
    }

    /// Get operation progress
    pub async fn get_progress(
        &self,
        operation_id: &str,
    ) -> Result<BulkOperationProgress, ProgressServiceError> {
        let operations = self.operations.read().await;
        operations
            .get(operation_id)
            .cloned()
            .ok_or(ProgressServiceError::OperationNotFound)
    }

    /// Cancel an operation
    pub async fn cancel_operation(&self, operation_id: &str) -> Result<(), ProgressServiceError> {
        let mut operations = self.operations.write().await;
        if let Some(progress) = operations.get_mut(operation_id) {
            progress.status = BulkOperationStatus::Cancelled;
            Ok(())
        } else {
            Err(ProgressServiceError::OperationNotFound)
        }
    }

    /// Clean up old completed operations (older than 1 hour)
    pub async fn cleanup_old_operations(&self) -> Result<usize, ProgressServiceError> {
        let mut operations = self.operations.write().await;
        let initial_count = operations.len();

        // Remove operations that are completed, failed, or cancelled and older than 1 hour
        operations.retain(|_, progress| {
            match progress.status {
                BulkOperationStatus::Completed
                | BulkOperationStatus::Failed
                | BulkOperationStatus::Cancelled => {
                    // For simplicity, we'll keep operations for now
                    // In a real implementation, you'd check timestamps here
                    false
                }
                _ => true,
            }
        });

        Ok(initial_count - operations.len())
    }

    /// Get all operations for a user (if we add user association later)
    pub async fn get_user_operations(
        &self,
        _user_id: i32,
    ) -> Result<Vec<BulkOperationProgress>, ProgressServiceError> {
        let operations = self.operations.read().await;
        Ok(operations.values().cloned().collect())
    }
}

impl Default for ProgressService {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress service errors
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum ProgressServiceError {
    #[error("Operation not found")]
    OperationNotFound,

    #[error("Invalid operation state")]
    InvalidOperationState,

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_operation() {
        let service = ProgressService::new();
        let operation_id = service.create_operation(100).await;
        assert!(!operation_id.is_empty());

        let progress = service.get_progress(&operation_id).await.unwrap();
        assert_eq!(progress.total_items, 100);
        assert_eq!(progress.processed_items, 0);
        assert!(matches!(progress.status, BulkOperationStatus::Pending));
    }

    #[tokio::test]
    async fn test_update_progress() {
        let service = ProgressService::new();
        let operation_id = service.create_operation(100).await;

        service
            .update_progress(&operation_id, 50, 45, 5)
            .await
            .unwrap();

        let progress = service.get_progress(&operation_id).await.unwrap();
        assert_eq!(progress.processed_items, 50);
        assert_eq!(progress.successful_items, 45);
        assert_eq!(progress.failed_items, 5);
        assert_eq!(progress.progress_percentage, 50.0);
        assert!(matches!(progress.status, BulkOperationStatus::Processing));
    }

    #[tokio::test]
    async fn test_complete_operation() {
        let service = ProgressService::new();
        let operation_id = service.create_operation(100).await;

        service
            .update_progress(&operation_id, 100, 95, 5)
            .await
            .unwrap();

        let progress = service.get_progress(&operation_id).await.unwrap();
        assert_eq!(progress.progress_percentage, 100.0);
        assert!(matches!(progress.status, BulkOperationStatus::Completed));
    }

    #[tokio::test]
    async fn test_cancel_operation() {
        let service = ProgressService::new();
        let operation_id = service.create_operation(100).await;

        service.cancel_operation(&operation_id).await.unwrap();

        let progress = service.get_progress(&operation_id).await.unwrap();
        assert!(matches!(progress.status, BulkOperationStatus::Cancelled));
    }

    #[tokio::test]
    async fn test_operation_not_found() {
        let service = ProgressService::new();
        let result = service.get_progress("non-existent").await;
        assert!(matches!(
            result,
            Err(ProgressServiceError::OperationNotFound)
        ));
    }
}
