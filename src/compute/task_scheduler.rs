use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::compute::gpu_manager::GpuManager;
use crate::ai::model_loader::ModelLoader;
use crate::error::OmniTensorError;
use crate::metrics::MetricsCollector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTask {
    pub id: String,
    pub model_id: String,
    pub input_data: Vec<u8>,
    pub priority: u8,
    pub max_duration: Duration,
}

pub struct TaskResult {
    pub task_id: String,
    pub output: Vec<u8>,
    pub execution_time: Duration,
}

#[async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute(&self, task: ComputeTask) -> Result<TaskResult, OmniTensorError>;
}

pub struct TaskScheduler {
    queue: Arc<Mutex<VecDeque<ComputeTask>>>,
    gpu_manager: Arc<GpuManager>,
    model_loader: Arc<ModelLoader>,
    metrics: Arc<MetricsCollector>,
    max_concurrent_tasks: usize,
}

impl TaskScheduler {
    pub fn new(
        gpu_manager: Arc<GpuManager>,
        model_loader: Arc<ModelLoader>,
        metrics: Arc<MetricsCollector>,
        max_concurrent_tasks: usize,
    ) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            gpu_manager,
            model_loader,
            metrics,
            max_concurrent_tasks,
        }
    }

    pub async fn submit_task(&self, task: ComputeTask) -> Result<(), OmniTensorError> {
        let mut queue = self.queue.lock().map_err(|_| OmniTensorError::LockError)?;
        queue.push_back(task);
        self.metrics.increment_queued_tasks();
        Ok(())
    }

    pub async fn run(&self) {
        loop {
            let task = {
                let mut queue = self.queue.lock().unwrap();
                queue.pop_front()
            };

            if let Some(task) = task {
                if let Err(e) = self.process_task(task).await {
                    log::error!("Error processing task: {:?}", e);
                }
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    async fn process_task(&self, task: ComputeTask) -> Result<(), OmniTensorError> {
        let gpu = self.gpu_manager.acquire_gpu().await?;
        let model = self.model_loader.load_model(&task.model_id).await?;

        let start_time = Instant::now();
        let result = model.execute(task.input_data).await?;
        let execution_time = start_time.elapsed();

        self.metrics.record_task_execution(execution_time);

        if execution_time > task.max_duration {
            log::warn!("Task {} exceeded max duration", task.id);
            self.metrics.increment_overdue_tasks();
        }

        self.gpu_manager.release_gpu(gpu).await?;

        // Here you would typically send the result back to the client or to a result queue
        log::info!("Task {} completed in {:?}", task.id, execution_time);

        Ok(())
    }

    pub async fn get_queue_length(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        GpuManager {}
        #[async_trait]
        impl GpuManager for GpuManager {
            async fn acquire_gpu(&self) -> Result<String, OmniTensorError>;
            async fn release_gpu(&self, gpu_id: String) -> Result<(), OmniTensorError>;
        }
    }

    mock! {
        ModelLoader {}
        #[async_trait]
        impl ModelLoader for ModelLoader {
            async fn load_model(&self, model_id: &str) -> Result<Arc<dyn TaskExecutor>, OmniTensorError>;
        }
    }

    #[tokio::test]
    async fn test_submit_and_process_task() {
        let mut gpu_manager = MockGpuManager::new();
        gpu_manager
            .expect_acquire_gpu()
            .returning(|| Ok("gpu1".to_string()));
        gpu_manager
            .expect_release_gpu()
            .returning(|_| Ok(()));

        let mut model_loader = MockModelLoader::new();
        model_loader
            .expect_load_model()
            .returning(|_| Ok(Arc::new(MockTaskExecutor::new())));

        let metrics = Arc::new(MetricsCollector::new());

        let scheduler = TaskScheduler::new(
            Arc::new(gpu_manager),
            Arc::new(model_loader),
            metrics,
            4,
        );

        let task = ComputeTask {
            id: "task1".to_string(),
            model_id: "model1".to_string(),
            input_data: vec![1, 2, 3],
            priority: 1,
            max_duration: Duration::from_secs(60),
        };

        scheduler.submit_task(task).await.unwrap();
        assert_eq!(scheduler.get_queue_length().await, 1);

      
    }
}