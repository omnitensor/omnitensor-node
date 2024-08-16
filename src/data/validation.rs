use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use async_trait::async_trait;

use crate::models::DataItem;
use crate::storage::DataStore;
use crate::consensus::ConsensusManager;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Consensus not reached")]
    ConsensusFailure,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Unknown error occurred")]
    Unknown,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence: f64,
    pub validator_count: usize,
}

pub struct DataValidator {
    data_store: Arc<Mutex<dyn DataStore>>,
    consensus_manager: Arc<ConsensusManager>,
}

impl DataValidator {
    pub fn new(data_store: Arc<Mutex<dyn DataStore>>, consensus_manager: Arc<ConsensusManager>) -> Self {
        Self {
            data_store,
            consensus_manager,
        }
    }

    pub async fn validate_data(&self, data: DataItem) -> Result<ValidationResult, ValidationError> {
        if !self.is_valid_format(&data) {
            return Err(ValidationError::InvalidFormat);
        }

        let validation_result = self.reach_consensus(&data).await?;

        self.store_validation_result(&data, &validation_result).await?;

        Ok(validation_result)
    }

    fn is_valid_format(&self, data: &DataItem) -> bool {
        // Implement format validation logic
        match data {
            DataItem::Text(text) => !text.is_empty() && text.len() <= 1000,
            DataItem::Image(image_data) => image_data.len() > 0 && image_data.len() <= 10_000_000, // Max 10MB
            DataItem::Numeric(num) => *num >= 0.0 && *num <= 1.0,
        }
    }

    async fn reach_consensus(&self, data: &DataItem) -> Result<ValidationResult, ValidationError> {
        let consensus_result = self.consensus_manager.reach_consensus(data).await
            .map_err(|_| ValidationError::ConsensusFailure)?;

        Ok(ValidationResult {
            is_valid: consensus_result.is_valid,
            confidence: consensus_result.confidence,
            validator_count: consensus_result.validator_count,
        })
    }

    async fn store_validation_result(&self, data: &DataItem, result: &ValidationResult) -> Result<(), ValidationError> {
        let mut store = self.data_store.lock().await;
        store.store_validation_result(data.id(), result)
            .map_err(|e| ValidationError::DatabaseError(e.to_string()))
    }
}

#[async_trait]
pub trait DataStore: Send + Sync {
    async fn store_validation_result(&mut self, id: String, result: &ValidationResult) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        DataStore {}
        #[async_trait]
        impl DataStore for MockDataStore {
            async fn store_validation_result(&mut self, id: String, result: &ValidationResult) -> Result<(), String>;
        }
    }

    mock! {
        ConsensusManager {}
        impl ConsensusManager {
            fn new() -> Self;
            async fn reach_consensus(&self, data: &DataItem) -> Result<ValidationResult, ValidationError>;
        }
    }

    #[tokio::test]
    async fn test_validate_data_success() {
        let mut mock_store = MockDataStore::new();
        mock_store
            .expect_store_validation_result()
            .with(eq("test_id"), always())
            .returning(|_, _| Ok(()));

        let mut mock_consensus = MockConsensusManager::new();
        mock_consensus
            .expect_reach_consensus()
            .returning(|_| Ok(ValidationResult {
                is_valid: true,
                confidence: 0.95,
                validator_count: 10,
            }));

        let validator = DataValidator::new(
            Arc::new(Mutex::new(mock_store)),
            Arc::new(mock_consensus),
        );

        let result = validator.validate_data(DataItem::Text("Valid data".to_string())).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_valid);
    }

    #[tokio::test]
    async fn test_validate_data_invalid_format() {
        let mock_store = MockDataStore::new();
        let mock_consensus = MockConsensusManager::new();

        let validator = DataValidator::new(
            Arc::new(Mutex::new(mock_store)),
            Arc::new(mock_consensus),
        );

        let result = validator.validate_data(DataItem::Text("".to_string())).await;
        assert!(matches!(result, Err(ValidationError::InvalidFormat)));
    }

    #[tokio::test]
    async fn test_validate_data_consensus_failure() {
        let mock_store = MockDataStore::new();
        
        let mut mock_consensus = MockConsensusManager::new();
        mock_consensus
            .expect_reach_consensus()
            .returning(|_| Err(ValidationError::ConsensusFailure));

        let validator = DataValidator::new(
            Arc::new(Mutex::new(mock_store)),
            Arc::new(mock_consensus),
        );

        let result = validator.validate_data(DataItem::Text("Valid data".to_string())).await;
        assert!(matches!(result, Err(ValidationError::ConsensusFailure)));
    }
}