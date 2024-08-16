use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use tch::{CModule, Device};

use crate::config::AIConfig;
use crate::storage::ModelStorage;
use crate::errors::ModelError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub version: String,
    pub task_type: String,
    pub input_shape: Vec<i64>,
    pub output_shape: Vec<i64>,
}

pub struct ModelLoader {
    config: AIConfig,
    storage: Arc<dyn ModelStorage>,
    loaded_models: Arc<RwLock<HashMap<String, (CModule, ModelMetadata)>>>,
}

impl ModelLoader {
    pub fn new(config: AIConfig, storage: Arc<dyn ModelStorage>) -> Self {
        Self {
            config,
            storage,
            loaded_models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_model(&self, model_id: &str) -> Result<Arc<CModule>> {
        // Check if model is already loaded
        if let Some(model) = self.loaded_models.read().await.get(model_id) {
            return Ok(Arc::new(model.0.clone()));
        }

        // Load model from storage
        let model_path = self.storage.get_model_path(model_id).await
            .context("Failed to get model path")?;
        let metadata = self.load_metadata(&model_path)
            .context("Failed to load model metadata")?;

        let device = if self.config.use_cuda {
            Device::Cuda(0)
        } else {
            Device::Cpu
        };

        let model = CModule::load_on_device(&model_path, device)
            .context("Failed to load model")?;

        // Store loaded model
        self.loaded_models.write().await.insert(
            model_id.to_string(),
            (model.clone(), metadata)
        );

        Ok(Arc::new(model))
    }

    async fn load_metadata(&self, model_path: &Path) -> Result<ModelMetadata> {
        let metadata_path = model_path.with_extension("json");
        let metadata_content = tokio::fs::read_to_string(&metadata_path).await
            .context("Failed to read metadata file")?;
        
        serde_json::from_str(&metadata_content)
            .context("Failed to parse metadata JSON")
    }

    pub async fn unload_model(&self, model_id: &str) -> Result<()> {
        self.loaded_models.write().await.remove(model_id);
        Ok(())
    }

    pub async fn get_model_metadata(&self, model_id: &str) -> Result<ModelMetadata> {
        if let Some(model) = self.loaded_models.read().await.get(model_id) {
            Ok(model.1.clone())
        } else {
            Err(ModelError::NotLoaded(model_id.to_string()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        ModelStorage {}
        #[async_trait]
        impl ModelStorage for ModelStorage {
            async fn get_model_path(&self, model_id: &str) -> Result<PathBuf>;
        }
    }

    #[tokio::test]
    async fn test_load_model() {
        let mut mock_storage = MockModelStorage::new();
        mock_storage
            .expect_get_model_path()
            .with(eq("test_model"))
            .returning(|_| Ok(PathBuf::from("test_path")));

        let config = AIConfig { use_cuda: false };
        let loader = ModelLoader::new(config, Arc::new(mock_storage));

        // This test will fail if running on a system without a CPU-compatible model at "test_path"
         let result = loader.load_model("test_model").await;
        assert!(result.is_ok());
    }
}