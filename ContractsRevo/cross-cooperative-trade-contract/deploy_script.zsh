#!/bin/zsh
# Usage: ./deploy_script.zsh [network] [profile]
# Example: ./deploy_script.zsh testnet default
# Networks: testnet, mainnet
# Profiles: e.g., default, release

# Check if arguments are provided
if [ $# -ne 2 ]; then
    echo "Error: Invalid number of arguments."
    echo "Usage: ./deploy_script.zsh [network] [profile]"
    echo "Example: ./deploy_script.zsh testnet default"
    exit 1
fi

NETWORK=$1
PROFILE=$2

# Validate network
if [[ "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" ]]; then
    echo "Error: Invalid network. Use 'testnet' or 'mainnet'."
    exit 1
fi

# Set WASM path (assuming standard Soroban build output)
WASM_PATH="target/wasm32-unknown-unknown/release/cross_cooperative_trade_contract.wasm"

# Log file
LOG_FILE="deployment_log.txt"

# Function to log and echo
log() {
    echo "$1" | tee -a $LOG_FILE
}

# Start logging
echo "Starting deployment at $(date)" > $LOG_FILE
log "Network: $NETWORK"
log "Profile: $PROFILE"

# Build the contract
log "Building contract with profile $PROFILE..."
if ! stellar contract build --profile $PROFILE; then
    log "Error: Build failed."
    exit 1
fi
log "Build successful."

# Check if WASM exists
if [ ! -f "$WASM_PATH" ]; then
    log "Error: WASM file not found at $WASM_PATH"
    exit 1
fi

# Upload the WASM
log "Uploading WASM to $NETWORK..."
UPLOAD_OUTPUT=$(stellar contract upload --network $NETWORK --wasm $WASM_PATH --json)
if [ $? -ne 0 ]; then
    log "Error: Upload failed."
    log "Upload output: $UPLOAD_OUTPUT"
    exit 1
fi
log "Upload output: $UPLOAD_OUTPUT"

# Parse WASM hash
WASM_HASH=$(echo $UPLOAD_OUTPUT | jq -r '.wasm_hash')
if [ "$WASM_HASH" == "null" ] || [ -z "$WASM_HASH" ]; then
    log "Error: Failed to parse wasm_hash."
    exit 1
fi
log "WASM Hash: $WASM_HASH"

# Deploy the contract
log "Deploying contract on $NETWORK..."
DEPLOY_OUTPUT=$(stellar contract deploy --network $NETWORK --wasm-hash $WASM_HASH --json)
if [ $? -ne 0 ]; then
    log "Error: Deploy failed."
    log "Deploy output: $DEPLOY_OUTPUT"
    exit 1
fi
log "Deploy output: $DEPLOY_OUTPUT"

# Parse Contract ID
CONTRACT_ID=$(echo $DEPLOY_OUTPUT | jq -r '.contract_id')
if [ "$CONTRACT_ID" == "null" ] || [ -z "$CONTRACT_ID" ]; then
    log "Error: Failed to parse contract_id."
    exit 1
fi
log "Contract ID: $CONTRACT_ID"

log "Deployment completed successfully at $(date)"
