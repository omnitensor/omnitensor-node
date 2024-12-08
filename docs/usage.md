# OmniTensor Node Usage Guide

## Introduction

The OmniTensor Node is an essential component of the OmniTensor ecosystem. It enables decentralized computation, model inference, and data validation by interacting with the blockchain and AI models. This guide provides instructions on how to set up, run and interact with the OmniTensor Node.

## Prerequisites

1. **Rust**: Ensure you have Rust installed. You can install it via `rustup`:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Dependencies**: Install necessary system dependencies. For Ubuntu:
   ```bash
   sudo apt update && sudo apt install -y build-essential libssl-dev
   ```

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/OmniTensor/
   cd OmniTensor-Project/omnitensor-node
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. The binary will be available at:
   ```bash
   target/release/omnitensor-node
   ```

## Configuration

Create a configuration file:
```bash
mkdir -p config
cat > config/node_config.toml <<EOL
[node]
id = "node-001"
listen_address = "0.0.0.0:3030"

[logging]
level = "info"
EOL
```

## Running the Node

1. To start the node:
   ```bash
   ./target/release/omnitensor-node --config config/node_config.toml
   ```

2. Logs will display the node's activity, including block synchronization and AI task scheduling.

## Interaction with the Node

You can interact with the node using the CLI or through RPC. For example, to fetch the current status:
```bash
curl http://localhost:3030/status
```

## Troubleshooting

- **Port Issues**: Ensure no other service is running on port `3030`.
- **Dependency Errors**: Verify that all required libraries are installed and up-to-date.

## Advanced Features

- **Custom GPU Management**: Edit the GPU section in the configuration file to manage GPU resources.
- **Logging Levels**: Adjust the logging level in `node_config.toml` to `debug` for detailed logs.

For additional help, refer to the [OmniTensor documentation](https://docs.omnitensor.io).
