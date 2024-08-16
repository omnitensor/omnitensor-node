#!/usr/bin/env bash

# OmniTensor Node Setup Script
# This script sets up the environment for running an OmniTensor node.

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
OMNITENSOR_VERSION="0.1.0"
RUST_VERSION="1.67.0"
REQUIRED_PACKAGES="build-essential curl libssl-dev pkg-config"
NODE_USER="omnitensor"
NODE_HOME="/home/$NODE_USER"
CONFIG_DIR="$NODE_HOME/.omnitensor"
LOG_DIR="$NODE_HOME/logs"
DATA_DIR="$NODE_HOME/data"

# Function to log messages
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"
}

# Function to log errors
error() {
    echo -e "${RED}[$(date +'%Y-%m-%dT%H:%M:%S%z')] ERROR: $1${NC}" >&2
}

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
        exit 1
    fi
}

# Function to update system packages
update_system() {
    log "Updating system packages..."
    apt-get update && apt-get upgrade -y
}

# Function to install required packages
install_packages() {
    log "Installing required packages..."
    apt-get install -y $REQUIRED_PACKAGES
}

# Function to install Rust
install_rust() {
    log "Installing Rust..."
    if command -v rustc >/dev/null 2>&1; then
        log "Rust is already installed. Updating..."
        su - $NODE_USER -c "rustup update"
    else
        su - $NODE_USER -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain $RUST_VERSION"
    fi
}

# Function to create OmniTensor user
create_user() {
    log "Creating OmniTensor user..."
    if id "$NODE_USER" &>/dev/null; then
        log "User $NODE_USER already exists"
    else
        useradd -m -s /bin/bash $NODE_USER
        echo "$NODE_USER ALL=(ALL) NOPASSWD:ALL" | tee /etc/sudoers.d/$NODE_USER
    fi
}

# Function to set up directories
setup_directories() {
    log "Setting up directories..."
    mkdir -p $CONFIG_DIR $LOG_DIR $DATA_DIR
    chown -R $NODE_USER:$NODE_USER $CONFIG_DIR $LOG_DIR $DATA_DIR
}

# Function to download OmniTensor node binary
download_binary() {
    log "Downloading OmniTensor node binary..."
    # TODO: Replace with actual download URL when available
    local download_url="https://github.com/omnitensor/releases/download/v${OMNITENSOR_VERSION}/omnitensor-node-${OMNITENSOR_VERSION}-linux-amd64"
    wget "$download_url" -O "$NODE_HOME/omnitensor-node"
    chmod +x "$NODE_HOME/omnitensor-node"
    chown $NODE_USER:$NODE_USER "$NODE_HOME/omnitensor-node"
}

# Function to set up configuration
setup_config() {
    log "Setting up configuration..."
    # TODO: Replace with actual configuration setup when available
    echo "# OmniTensor Node Configuration" > "$CONFIG_DIR/config.toml"
    echo "log_level = \"info\"" >> "$CONFIG_DIR/config.toml"
    echo "data_dir = \"$DATA_DIR\"" >> "$CONFIG_DIR/config.toml"
    chown $NODE_USER:$NODE_USER "$CONFIG_DIR/config.toml"
}

# Function to set up systemd service
setup_service() {
    log "Setting up systemd service..."
    cat << EOF > /etc/systemd/system/omnitensor-node.service
[Unit]
Description=OmniTensor Node
After=network.target

[Service]
User=$NODE_USER
ExecStart=$NODE_HOME/omnitensor-node --config $CONFIG_DIR/config.toml
Restart=on-failure
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable omnitensor-node.service
}

# Function to start the node
start_node() {
    log "Starting OmniTensor node..."
    systemctl start omnitensor-node.service
}

# Main function
main() {
    log "Starting OmniTensor node setup..."

    check_root
    update_system
    install_packages
    create_user
    install_rust
    setup_directories
    download_binary
    setup_config
    setup_service
    start_node

    log "OmniTensor node setup completed successfully!"
    log "You can check the node status with: systemctl status omnitensor-node"
    log "Node logs are available at: $LOG_DIR"
}

# Run main function
main