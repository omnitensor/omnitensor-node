use tokio;
use tracing::{info, error};
use clap::{App, Arg};
use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
mod network;
mod consensus;
mod storage;
mod compute;

use crate::config::Config;
use crate::network::Network;
use crate::network::Message as NetworkMessage;
use crate::consensus::Consensus;
use crate::consensus::{Block, Transaction};
use crate::storage::Storage;
use crate::compute::ComputeManager;
use crate::compute::{Event as ComputeEvent, Task, TaskStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let matches = App::new("OmniTensor Node")
        .version("0.1.0")
        .author("OmniTensor Team")
        .about("Decentralized AI Infrastructure Node")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .get_matches();

    // Load configuration
    let config_path = matches.value_of("config").unwrap_or("config/default.toml");
    let config = Config::from_file(config_path)?;

    info!("Starting OmniTensor node with config: {}", config_path);

    // Initialize components
    let storage = Arc::new(Mutex::new(Storage::new(&config.storage)?));
    let network = Arc::new(Network::new(&config.network)?);
    let consensus = Arc::new(Consensus::new(&config.consensus, network.clone(), storage.clone())?);
    let compute_manager = Arc::new(ComputeManager::new(&config.compute)?);

    // Start network services
    network.start().await?;

    // Start consensus engine
    consensus.start().await?;

    // Start compute manager
    compute_manager.start().await?;

    // Main event loop
    loop {
        tokio::select! {
            Some(event) = network.next_event() => {
                match event {
                    Ok(network_event) => {
                        // Handle network events
                        if let Err(e) = handle_network_event(network_event, &consensus, &compute_manager).await {
                            error!("Error handling network event: {}", e);
                        }
                    },
                    Err(e) => error!("Network error: {}", e),
                }
            }
            Some(event) = consensus.next_event() => {
                match event {
                    Ok(consensus_event) => {
                        // Handle consensus events
                        if let Err(e) = handle_consensus_event(consensus_event, &network, &compute_manager).await {
                            error!("Error handling consensus event: {}", e);
                        }
                    },
                    Err(e) => error!("Consensus error: {}", e),
                }
            }
            Some(event) = compute_manager.next_event() => {
                match event {
                    Ok(compute_event) => {
                        // Handle compute events
                        if let Err(e) = handle_compute_event(compute_event, &network, &consensus).await {
                            error!("Error handling compute event: {}", e);
                        }
                    },
                    Err(e) => error!("Compute error: {}", e),
                }
            }
            else => break,
        }
    }

    // Graceful shutdown
    info!("Shutting down OmniTensor node");
    compute_manager.stop().await?;
    consensus.stop().await?;
    network.stop().await?;

    Ok(())
}

async fn handle_network_event(
    event: network::Event,
    consensus: &Arc<Consensus>,
    compute_manager: &Arc<ComputeManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement network event handling
    Ok(())
}

async fn handle_consensus_event(
    event: consensus::Event,
    network: &Arc<Network>,
    compute_manager: &Arc<ComputeManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement consensus event handling
    Ok(())
}

async fn handle_compute_event(
    event: ComputeEvent,
    network: &Arc<Network>,
    consensus: &Arc<Consensus>,
) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        ComputeEvent::TaskCompleted(task) => {
            info!("Task completed: {}", task.id);
            
            // Update task status in local storage
            consensus.storage.lock().await.update_task_status(&task.id, TaskStatus::Completed)?;
            
            // Create a transaction for the completed task
            let transaction = Transaction::new_task_completion(task.id, task.result_hash);
            
            // Submit the transaction to the consensus layer
            consensus.submit_transaction(transaction).await?;
            
            // Notify the network about the completed task
            let message = NetworkMessage::TaskCompleted { 
                task_id: task.id, 
                result_hash: task.result_hash 
            };
            network.broadcast(message).await?;
        },
        ComputeEvent::TaskFailed(task_id, error) => {
            error!("Task failed: {}. Error: {}", task_id, error);
            
            // Update task status in local storage
            consensus.storage.lock().await.update_task_status(&task_id, TaskStatus::Failed)?;
            
            // Create a transaction for the failed task
            let transaction = Transaction::new_task_failure(task_id, error);
            
            // Submit the transaction to the consensus layer
            consensus.submit_transaction(transaction).await?;
            
            // Notify the network about the failed task
            let message = NetworkMessage::TaskFailed { task_id, error };
            network.broadcast(message).await?;
        },
        ComputeEvent::NewTaskReceived(task) => {
            info!("New task received: {}", task.id);
            
            // Verify if the node has capacity to handle the task
            if compute_manager.has_capacity() {
                // Accept the task
                compute_manager.accept_task(task).await?;
                
                // Update task status in local storage
                consensus.storage.lock().await.update_task_status(&task.id, TaskStatus::InProgress)?;
                
                // Notify the network that we've accepted the task
                let message = NetworkMessage::TaskAccepted { task_id: task.id };
                network.broadcast(message).await?;
            } else {
                // Reject the task if we don't have capacity
                let message = NetworkMessage::TaskRejected { 
                    task_id: task.id, 
                    reason: "No capacity".to_string() 
                };
                network.broadcast(message).await?;
            }
        },
        ComputeEvent::ResourceUsageUpdate(usage) => {
            // Periodically update the network about our resource usage
            let message = NetworkMessage::ResourceUsage { 
                node_id: network.node_id(), 
                cpu_usage: usage.cpu, 
                memory_usage: usage.memory, 
                gpu_usage: usage.gpu 
            };
            network.broadcast(message).await?;
            
            // If resource usage is high, consider offloading tasks
            if usage.is_high() {
                compute_manager.consider_offloading().await?;
            }
        },
        ComputeEvent::ModelUpdated(model_id, new_version) => {
            info!("Model updated: {} to version {}", model_id, new_version);
            
            // Create a transaction for the model update
            let transaction = Transaction::new_model_update(model_id, new_version);
            
            // Submit the transaction to the consensus layer
            consensus.submit_transaction(transaction).await?;
            
            // Notify the network about the model update
            let message = NetworkMessage::ModelUpdated { model_id, new_version };
            network.broadcast(message).await?;
        },
    }

    Ok(())
}