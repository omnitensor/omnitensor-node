//! Example: Hello World Node
//!
//! This is a basic example to demonstrate how to use the OmniTensor Node.

use std::env;

fn main() {
    // Display a welcome message
    println!("Welcome to the OmniTensor Node!");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("Received arguments: {:?}", &args[1..]);
    } else {
        println!("No arguments provided.");
    }

    // Simulate a node action
    println!("Initializing node components...");
    simulate_node_startup();

    println!("Node is now running. Use Ctrl+C to stop.");
}

fn simulate_node_startup() {
    use std::{thread, time};

    let startup_phases = [
        "Loading configuration",
        "Initializing networking",
        "Connecting to peers",
        "Synchronizing blockchain",
        "Starting AI compute engine",
    ];

    for phase in &startup_phases {
        println!("{}...", phase);
        thread::sleep(time::Duration::from_secs(1)); // Simulate delay
    }

    println!("All components initialized successfully!");
}
