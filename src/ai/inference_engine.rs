use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use tch::{Device, Tensor, nn};
use crate::models::{ModelRegistry, ModelType};
use crate::utils::tensor_utils::TensorConversion;
use crate::config::AIConfig;

#[derive(Clone)]
pub struct InferenceEngine {
    model_registry: Arc<ModelRegistry>,
    config: Arc<AIConfig>,
    device: Device,
}

#[derive(Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_id: String,
    pub input: Vec<f32>,
    pub params: Option<InferenceParams>,
}

#[derive(Serialize, Deserialize)]
pub struct InferenceParams {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct InferenceResponse {
    pub output: Vec<f32>,
    pub latency: f64,
}

impl InferenceEngine {
    pub fn new(model_registry: Arc<ModelRegistry>, config: Arc<AIConfig>) -> Self {
        let device = if cuda::is_available() { Device::Cuda(0) } else { Device::Cpu };
        Self { model_registry, config, device }
    }

    pub async fn run_inference(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let model = self.model_registry.get_model(&request.model_id)
            .context("Failed to get model from registry")?;

        let input_tensor = Tensor::of_slice(&request.input).to(self.device);
        
        let start_time = std::time::Instant::now();
        
        let output_tensor = match model.model_type() {
            ModelType::Transformer => self.run_transformer_inference(model, input_tensor, request.params).await?,
            ModelType::CNN => self.run_cnn_inference(model, input_tensor).await?,
            // Add more model types as needed
        };

        let latency = start_time.elapsed().as_secs_f64();

        let output = output_tensor.to_vec1::<f32>()?;

        Ok(InferenceResponse { output, latency })
    }

    async fn run_transformer_inference(
        &self,
        model: Arc<dyn nn::Module>,
        input: Tensor,
        params: Option<InferenceParams>
    ) -> Result<Tensor> {
        let params = params.unwrap_or_default();
        let temperature = params.temperature.unwrap_or(self.config.default_temperature);
        let top_p = params.top_p.unwrap_or(self.config.default_top_p);
        let max_tokens = params.max_tokens.unwrap_or(self.config.default_max_tokens);

        // Assuming the model is wrapped in no_grad for inference
        let output = tch::no_grad(|| {
            model.forward_t(&input, false)
                .context("Failed to run transformer inference")
        })?;

        // Apply temperature scaling and top-p sampling
        let scaled_output = output / temperature;
        let sampled_output = self.top_p_sampling(scaled_output, top_p, max_tokens)?;

        Ok(sampled_output)
    }

    async fn run_cnn_inference(&self, model: Arc<dyn nn::Module>, input: Tensor) -> Result<Tensor> {
        tch::no_grad(|| {
            model.forward_t(&input, false)
                .context("Failed to run CNN inference")
        })
    }

    fn top_p_sampling(&self, logits: Tensor, p: f32, max_tokens: i64) -> Result<Tensor> {
        // Implement top-p (nucleus) sampling
        let sorted_logits = logits.argsort(-1, true);
        let cumulative_probs = sorted_logits.softmax(-1, tch::Kind::Float).cumsum(-1, tch::Kind::Float);
        let sorted_indices_to_remove = cumulative_probs > p;
        let indices_to_remove = sorted_indices_to_remove.scatter(1, sorted_logits, sorted_indices_to_remove);
        
        let filtered_logits = logits.masked_fill(&indices_to_remove, f64::NEG_INFINITY);
        let sampled_tokens = filtered_logits.multinomial(max_tokens, true);

        Ok(sampled_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MockModel;

    #[tokio::test]
    async fn test_inference_engine() {
        let config = Arc::new(AIConfig::default());
        let model_registry = Arc::new(ModelRegistry::new());
        let mock_model = Arc::new(MockModel::new());
        model_registry.register("test_model".to_string(), mock_model.clone()).unwrap();

        let engine = InferenceEngine::new(model_registry, config);

        let request = InferenceRequest {
            model_id: "test_model".to_string(),
            input: vec![1.0, 2.0, 3.0],
            params: None,
        };

        let response = engine.run_inference(request).await.unwrap();

        assert_eq!(response.output.len(), 3);
        assert!(response.latency > 0.0);
    }
}