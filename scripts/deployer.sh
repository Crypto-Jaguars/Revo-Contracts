#!/bin/bash

# Function to display usage
usage() {
    echo "Usage: $0 --source <source_key> --wasm <wasm_file_name>"
    echo ""
    echo "Arguments:"
    echo "  --source    Stellar source key (e.g., kennyv4)"
    echo "  --wasm      WASM file name without .wasm extension (e.g., product_auction_contract)"
    echo ""
    echo "Available WASM files:"
    echo "  - product_auction_contract"
    echo "  - purchase_review_contract"
    echo "  - market_demand_forecasting_contract"
    echo "  - loyalty_token_contract"
    echo "  - land_leasing_contract"
    echo "  - farmer_insurance_contract"
    echo "  - environmental_impact_tracking"
    echo "  - csa_membership_contract"
    echo "  - crowdfunding_farmer_contract"
    echo "  - cross_cooperative_trade_contract"
    echo "  - supply_chain_tracking_contract"
    echo "  - micro_lending"
    echo "  - farmer_token_contract"
    echo "  - agricultural_auction_contract"
    echo "  - agricultural_quality_contract"
    echo "  - agricultural_training_contract"
    echo "  - certificate_management_contract"
    echo "  - commodity_token_contract"
    echo "  - crop_yield_prediction"
    echo "  - equipment_rental_contract"
    echo "  - farmer_liquidity_pool_contract"
    echo "  - rating_system_contract"
    echo "  - transaction_nft_contract"
    echo "  - water_management_contract"
    echo "  - a_lp_token_contract"
    echo ""
    echo "Example: $0 --source kennyv4 --wasm product_auction_contract"
    exit 1
}

# Parse command line arguments
SOURCE=""
WASM_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --source)
            SOURCE="$2"
            shift 2
            ;;
        --wasm)
            WASM_FILE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option $1"
            usage
            ;;
    esac
done

# Validate arguments
if [ -z "$SOURCE" ] || [ -z "$WASM_FILE" ]; then
    echo "Error: Both --source and --wasm arguments are required"
    usage
fi

log() {
    echo -e "\n\033[1;32m[LOG] $1\033[0m"
}

error() {
    echo -e "\033[1;31m[ERROR] $1\033[0m"
}

log "Checking Rust target..."
if ! rustup target list | grep -q "wasm32v1-none (installed)"; then
    log "Adding wasm32v1-none target..."
    rustup target add wasm32v1-none
    if [ $? -ne 0 ]; then
        error "Failed to add wasm32v1-none target"
        exit 1
    fi
fi

# Build the contracts
log "Building contracts..."
cargo build --target wasm32v1-none --release
if [ $? -ne 0 ]; then
    error "Build failed"
    exit 1
fi
# Check if WASM file exists
WASM_PATH="target/wasm32v1-none/release/${WASM_FILE}.wasm"
if [ ! -f "$WASM_PATH" ]; then
    echo "Error: WASM file not found: $WASM_PATH"
    echo "Make sure the contract has been built and the file name is correct."
    exit 1
fi

# Dynamic contract deployment
log "Uploading $WASM_FILE contract..."
upload_output=$(stellar contract upload \
  --network testnet \
  --source $SOURCE \
  --wasm $WASM_PATH)
echo "$upload_output"

if [ $? -ne 0 ]; then
    error "Failed to upload contract"
    exit 1
fi

log "Deploying $WASM_FILE contract with wasm hash: $upload_output"

# Create a proper alias 
alias_name=$(echo "${WASM_FILE:0:1}" | tr '[:lower:]' '[:upper:]')${WASM_FILE:1}Contract

deploy_output=$(stellar contract deploy \
  --wasm-hash $upload_output \
  --source $SOURCE \
  --network testnet \
  --alias $alias_name)

deploy_exit_code=$?

if [ $deploy_exit_code -ne 0 ]; then
    error "Failed to deploy contract"
    exit 1
fi

if [ -z "$deploy_output" ]; then
    error "Deploy command succeeded but returned empty contract ID"
    exit 1
fi

echo "[LOG] $WASM_FILE contract ID: $deploy_output"
log "Successfully deployed $WASM_FILE contract!"
