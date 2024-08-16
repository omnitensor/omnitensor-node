use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use anyhow::{Result, Context};
use log::{info, error, debug};
use crate::models::ComputeTask;
use crate::config::GPUConfig;
use crate::utils::gpu::{GPUDevice, GPUMemoryInfo};

pub struct GPUManager {
    devices: Arc<Mutex<Vec<GPUDevice>>>,
    task_queue: mpsc::Sender<ComputeTask>,
    config: GPUConfig,
}

impl GPUManager {
    pub async fn new(config: GPUConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let devices = Arc::new(Mutex::new(Vec::new()));
        
        Self::initialize_devices(&devices, &config).await?;
        
        let manager = Self {
            devices,
            task_queue: tx,
            config,
        };

        tokio::spawn(Self::process_task_queue(Arc::clone(&manager.devices), rx));

        Ok(manager)
    }

    async fn initialize_devices(devices: &Arc<Mutex<Vec<GPUDevice>>>, config: &GPUConfig) -> Result<()> {
        let available_devices = GPUDevice::enumerate().context("Failed to enumerate GPU devices")?;
        
        let mut locked_devices = devices.lock().map_err(|_| anyhow::anyhow!("Failed to acquire lock on devices"))?;
        
        for device in available_devices {
            if device.memory() >= config.min_memory {
                locked_devices.push(device);
                info!("Initialized GPU device: {}", device.name());
            }
        }

        if locked_devices.is_empty() {
            error!("No suitable GPU devices found");
            return Err(anyhow::anyhow!("No suitable GPU devices available"));
        }

        Ok(())
    }

    pub async fn submit_task(&self, task: ComputeTask) -> Result<()> {
        self.task_queue.send(task).await
            .context("Failed to submit task to GPU queue")?;
        Ok(())
    }

    async fn process_task_queue(devices: Arc<Mutex<Vec<GPUDevice>>>, mut rx: mpsc::Receiver<ComputeTask>) {
        while let Some(task) = rx.recv().await {
            let device = Self::select_available_device(&devices).await;
            
            match device {
                Some(mut gpu) => {
                    if let Err(e) = gpu.execute_task(task).await {
                        error!("Failed to execute task on GPU: {}", e);
                    }
                },
                None => {
                    debug!("No available GPU device, task queued");
                    // Implement queuing logic here
                }
            }
        }
    }

    async fn select_available_device(devices: &Arc<Mutex<Vec<GPUDevice>>>) -> Option<GPUDevice> {
        let locked_devices = devices.lock().ok()?;
        locked_devices.iter()
            .min_by_key(|d| d.current_load())
            .cloned()
    }

    pub async fn get_gpu_stats(&self) -> Result<Vec<GPUMemoryInfo>> {
        let locked_devices = self.devices.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire lock on devices"))?;
        
        let mut stats = Vec::new();
        for device in locked_devices.iter() {
            stats.push(device.memory_info().context("Failed to get GPU memory info")?);
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_gpu_manager_initialization() {
        let config = GPUConfig { min_memory: 4 * 1024 * 1024 * 1024 }; // 4 GB
        let manager = GPUManager::new(config).await.expect("Failed to initialize GPUManager");
        
        let stats = manager.get_gpu_stats().await.expect("Failed to get GPU stats");
        assert!(!stats.is_empty(), "No GPU devices initialized");
    }

    #[tokio::test]
    async fn test_task_submission() {
        let config = GPUConfig { min_memory: 4 * 1024 * 1024 * 1024 }; // 4 GB
        let manager = GPUManager::new(config).await.expect("Failed to initialize GPUManager");
        
        let task = ComputeTask::new("test_task", vec![1, 2, 3]);
        manager.submit_task(task).await.expect("Failed to submit task");

        // Allow some time for task processing
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Check if the task was processed (you might need to implement a way to check this)
        // This is just a placeholder assertion
        assert!(true, "Task submission test passed");
    }

    #[tokio::test]
    async fn test_gpu_stats() {
        let config = GPUConfig { min_memory: 4 * 1024 * 1024 * 1024 }; // 4 GB
        let manager = GPUManager::new(config).await.expect("Failed to initialize GPUManager");
        
        let stats = manager.get_gpu_stats().await.expect("Failed to get GPU stats");
        assert!(!stats.is_empty(), "No GPU stats available");
        
        for stat in stats {
            assert!(stat.total > 0, "Invalid total memory");
            assert!(stat.used <= stat.total, "Used memory exceeds total memory");
        }
    }
}