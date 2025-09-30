#!/bin/zsh

# Product Auction Contract Deployment Script
# Usage: ./deploy_product_auction.zsh [network] [profile]
# Example: ./deploy_product_auction.zsh testnet default

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
CONTRACT_NAME="product-auction-contract"
CONTRACT_DIR="/Users/villarley/Documents/GitHub/Revo-Contracts/ContractsRevo/product-auction-contract"
WASM_PATH="${CONTRACT_DIR}/target/wasm32-unknown-unknown/release/product_auction_contract.wasm"
LOG_DIR="${CONTRACT_DIR}/logs"
DEPLOYMENT_LOG="${LOG_DIR}/deployment_$(date +%Y%m%d_%H%M%S).log"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$DEPLOYMENT_LOG"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$DEPLOYMENT_LOG"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$DEPLOYMENT_LOG"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$DEPLOYMENT_LOG"
}

print_header() {
    echo -e "${CYAN}================================${NC}" | tee -a "$DEPLOYMENT_LOG"
    echo -e "${CYAN}$1${NC}" | tee -a "$DEPLOYMENT_LOG"
    echo -e "${CYAN}================================${NC}" | tee -a "$DEPLOYMENT_LOG"
}

# Function to log with timestamp
log_with_timestamp() {
    echo "[$(date -u +"%Y-%m-%d %H:%M:%S UTC")] $1" | tee -a "$DEPLOYMENT_LOG"
}

# Check if required tools are installed
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check for stellar CLI
    if ! command -v stellar &> /dev/null; then
        print_error "Stellar CLI is not installed. Please install it first:"
        echo "cargo install stellar-cli"
        exit 1
    fi
    
    # Check for jq
    if ! command -v jq &> /dev/null; then
        print_error "jq is not installed. Please install it first:"
        echo "brew install jq  # macOS"
        echo "apt-get install jq  # Ubuntu/Debian"
        exit 1
    fi
    
    # Check for cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Validate parameters
validate_parameters() {
    local network="$1"
    local profile="$2"
    
    # Validate network parameter
    if [[ -z "$network" ]]; then
        print_error "Network parameter is required"
        print_usage
        exit 1
    fi
    
    if [[ "$network" != "testnet" && "$network" != "mainnet" ]]; then
        print_error "Invalid network: $network. Must be 'testnet' or 'mainnet'"
        print_usage
        exit 1
    fi
    
    # Set default profile if not provided
    if [[ -z "$profile" ]]; then
        profile="default"
        print_warning "No profile specified, using default profile"
    fi
    
    # Validate profile exists
    if ! stellar config keys list --profile "$profile" &> /dev/null; then
        print_error "Profile '$profile' does not exist or is not configured"
        echo "Available profiles:"
        stellar config keys list
        exit 1
    fi
    
    print_success "Parameters validated: network=$network, profile=$profile"
}

# Setup logging directory
setup_logging() {
    mkdir -p "$LOG_DIR"
    print_status "Logging to: $DEPLOYMENT_LOG"
    
    # Initialize log file
    echo "Product Auction Contract Deployment Log" > "$DEPLOYMENT_LOG"
    echo "Started at: $(date -u +"%Y-%m-%d %H:%M:%S UTC")" >> "$DEPLOYMENT_LOG"
    echo "Network: $NETWORK" >> "$DEPLOYMENT_LOG"
    echo "Profile: $PROFILE" >> "$DEPLOYMENT_LOG"
    echo "========================================" >> "$DEPLOYMENT_LOG"
}

# Build the contract
build_contract() {
    print_header "BUILDING CONTRACT"
    
    cd "$CONTRACT_DIR"
    
    print_status "Building contract with profile: $PROFILE"
    log_with_timestamp "Starting contract build"
    
    # Build using stellar contract build
    if ! stellar contract build --profile "$PROFILE" 2>&1 | tee -a "$DEPLOYMENT_LOG"; then
        print_error "Contract build failed"
        log_with_timestamp "Contract build failed"
        exit 1
    fi
    
    # Verify WASM file exists
    if [[ ! -f "$WASM_PATH" ]]; then
        print_error "WASM file not found at expected path: $WASM_PATH"
        print_status "Available WASM files:"
        find "$CONTRACT_DIR/target" -name "*.wasm" 2>/dev/null | tee -a "$DEPLOYMENT_LOG" || true
        exit 1
    fi
    
    local wasm_size=$(ls -lh "$WASM_PATH" | awk '{print $5}')
    print_success "Contract built successfully"
    print_status "WASM file: $WASM_PATH"
    print_status "WASM size: $wasm_size"
    log_with_timestamp "Contract build completed successfully"
}

# Upload the contract
upload_contract() {
    print_header "UPLOADING CONTRACT"
    
    print_status "Uploading contract to $NETWORK network"
    log_with_timestamp "Starting contract upload"
    
    # Upload the contract and capture output
    local upload_output
    if ! upload_output=$(stellar contract upload --network "$NETWORK" --wasm "$WASM_PATH" --json 2>&1); then
        print_error "Contract upload failed"
        echo "$upload_output" | tee -a "$DEPLOYMENT_LOG"
        log_with_timestamp "Contract upload failed"
        exit 1
    fi
    
    # Parse WASM hash from output
    WASM_HASH=$(echo "$upload_output" | jq -r '.wasm_hash // empty')
    
    if [[ -z "$WASM_HASH" || "$WASM_HASH" == "null" ]]; then
        print_error "Failed to extract WASM hash from upload output"
        echo "Upload output: $upload_output" | tee -a "$DEPLOYMENT_LOG"
        log_with_timestamp "Failed to extract WASM hash"
        exit 1
    fi
    
    print_success "Contract uploaded successfully"
    print_status "WASM Hash: $WASM_HASH"
    log_with_timestamp "Contract upload completed - WASM Hash: $WASM_HASH"
    
    # Save upload output to log
    echo "Upload Output:" >> "$DEPLOYMENT_LOG"
    echo "$upload_output" >> "$DEPLOYMENT_LOG"
    echo "" >> "$DEPLOYMENT_LOG"
}

# Deploy the contract
deploy_contract() {
    print_header "DEPLOYING CONTRACT"
    
    print_status "Deploying contract to $NETWORK network"
    log_with_timestamp "Starting contract deployment"
    
    # Deploy the contract and capture output
    local deploy_output
    if ! deploy_output=$(stellar contract deploy --network "$NETWORK" --wasm-hash "$WASM_HASH" --json 2>&1); then
        print_error "Contract deployment failed"
        echo "$deploy_output" | tee -a "$DEPLOYMENT_LOG"
        log_with_timestamp "Contract deployment failed"
        exit 1
    fi
    
    # Parse contract ID from output
    CONTRACT_ID=$(echo "$deploy_output" | jq -r '.contract_id // empty')
    
    if [[ -z "$CONTRACT_ID" || "$CONTRACT_ID" == "null" ]]; then
        print_error "Failed to extract contract ID from deployment output"
        echo "Deployment output: $deploy_output" | tee -a "$DEPLOYMENT_LOG"
        log_with_timestamp "Failed to extract contract ID"
        exit 1
    fi
    
    print_success "Contract deployed successfully"
    print_status "Contract ID: $CONTRACT_ID"
    log_with_timestamp "Contract deployment completed - Contract ID: $CONTRACT_ID"
    
    # Save deployment output to log
    echo "Deployment Output:" >> "$DEPLOYMENT_LOG"
    echo "$deploy_output" >> "$DEPLOYMENT_LOG"
    echo "" >> "$DEPLOYMENT_LOG"
}

# Save deployment results
save_deployment_results() {
    print_header "SAVING DEPLOYMENT RESULTS"
    
    local results_file="${LOG_DIR}/deployment_results.json"
    local timestamp=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
    
    # Create JSON results file
    cat > "$results_file" << EOF
{
  "contract_name": "$CONTRACT_NAME",
  "network": "$NETWORK",
  "profile": "$PROFILE",
  "wasm_hash": "$WASM_HASH",
  "contract_id": "$CONTRACT_ID",
  "deployment_timestamp": "$timestamp",
  "wasm_path": "$WASM_PATH",
  "deployment_log": "$DEPLOYMENT_LOG"
}
EOF
    
    print_success "Deployment results saved to: $results_file"
    
    # Also save a simple summary file
    local summary_file="${LOG_DIR}/latest_deployment.txt"
    cat > "$summary_file" << EOF
Product Auction Contract Deployment Summary
===========================================
Contract: $CONTRACT_NAME
Network: $NETWORK
Profile: $PROFILE
WASM Hash: $WASM_HASH
Contract ID: $CONTRACT_ID
Deployed: $timestamp
Log File: $DEPLOYMENT_LOG
EOF
    
    print_success "Deployment summary saved to: $summary_file"
    log_with_timestamp "Deployment results saved"
}

# Display final results
display_results() {
    print_header "DEPLOYMENT COMPLETED"
    
    echo -e "${GREEN}✅ Contract deployed successfully!${NC}"
    echo ""
    echo -e "${CYAN}Deployment Details:${NC}"
    echo "  Contract: $CONTRACT_NAME"
    echo "  Network: $NETWORK"
    echo "  Profile: $PROFILE"
    echo "  WASM Hash: $WASM_HASH"
    echo "  Contract ID: $CONTRACT_ID"
    echo ""
    echo -e "${CYAN}Files Created:${NC}"
    echo "  Deployment Log: $DEPLOYMENT_LOG"
    echo "  Results JSON: ${LOG_DIR}/deployment_results.json"
    echo "  Summary: ${LOG_DIR}/latest_deployment.txt"
    echo ""
    echo -e "${YELLOW}Next Steps:${NC}"
    echo "  1. Verify deployment on Stellar Explorer"
    echo "  2. Test contract functionality"
    echo "  3. Initialize contract if required"
    echo ""
    
    # Network-specific instructions
    if [[ "$NETWORK" == "testnet" ]]; then
        echo -e "${BLUE}Testnet Explorer:${NC} https://testnet.stellar.org/explorer"
    else
        echo -e "${BLUE}Mainnet Explorer:${NC} https://stellar.org/explorer"
    fi
    
    log_with_timestamp "Deployment process completed successfully"
}

# Print usage information
print_usage() {
    echo "Usage: ./deploy_product_auction.zsh [network] [profile]"
    echo ""
    echo "Arguments:"
    echo "  network    - Target network (testnet or mainnet)"
    echo "  profile    - Stellar profile to use (optional, defaults to 'default')"
    echo ""
    echo "Examples:"
    echo "  ./deploy_product_auction.zsh testnet                    # Deploy to testnet with default profile"
    echo "  ./deploy_product_auction.zsh testnet my_profile         # Deploy to testnet with custom profile"
    echo "  ./deploy_product_auction.zsh mainnet production         # Deploy to mainnet with production profile"
    echo ""
    echo "Prerequisites:"
    echo "  - Stellar CLI installed (cargo install stellar-cli)"
    echo "  - jq installed (brew install jq)"
    echo "  - Valid Stellar profile configured"
    echo ""
    echo "The script will:"
    echo "  1. Build the product auction contract"
    echo "  2. Upload the WASM to the specified network"
    echo "  3. Deploy the contract and capture the contract ID"
    echo "  4. Save deployment logs and results"
}

# Cleanup function
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        print_error "Deployment failed with exit code: $exit_code"
        log_with_timestamp "Deployment failed with exit code: $exit_code"
    fi
    exit $exit_code
}

# Set up error handling
trap cleanup EXIT

# Main function
main() {
    local network="$1"
    local profile="$2"
    
    # Check if help is requested
    if [[ "$1" == "-h" || "$1" == "--help" ]]; then
        print_usage
        exit 0
    fi
    
    # Initialize logging
    setup_logging
    
    # Validate prerequisites and parameters
    check_prerequisites
    validate_parameters "$network" "$profile"
    
    # Set global variables
    NETWORK="$network"
    PROFILE="$profile"
    
    # Execute deployment steps
    build_contract
    upload_contract
    deploy_contract
    save_deployment_results
    display_results
}

# Run main function with all arguments
main "$@"
