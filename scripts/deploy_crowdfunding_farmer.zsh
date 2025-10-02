#!/bin/zsh

# Crowdfunding Farmer Contract Deployment Script
# Usage: ./deploy_crowdfunding_farmer.zsh [network] [identity]
# Example: ./deploy_crowdfunding_farmer.zsh testnet default

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
CONTRACT_NAME="crowdfunding-farmer-contract"
SCRIPT_DIR="${0:A:h}"
PROJECT_ROOT="${SCRIPT_DIR:h}"
CONTRACT_DIR="${PROJECT_ROOT}/ContractsRevo/crowdfunding-farmer-contract"
# Modern Stellar CLI uses workspace root target directory with wasm32v1-none
WASM_PATH="${PROJECT_ROOT}/target/wasm32v1-none/release/crowdfunding_farmer_contract.wasm"
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

    # Check for cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi

    # Check for jq (needed for JSON parsing if using --json flag)
    if ! command -v jq &> /dev/null; then
        print_warning "jq is not installed. JSON output parsing may not work properly."
        print_status "Install jq with: sudo apt-get install jq (Linux) or brew install jq (macOS)"
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
        print_warning "No identity specified, using default identity"
    fi

    # Validate profile exists
    if ! stellar keys ls 2>&1 | grep -q "$profile"; then
        print_error "Identity '$profile' does not exist or is not configured"
        echo "Available identities:"
        stellar keys ls
        exit 1
    fi

    print_success "Parameters validated: network=$network, identity=$profile"
}

# Setup logging directory
setup_logging() {
    mkdir -p "$LOG_DIR"
    print_status "Logging to: $DEPLOYMENT_LOG"

    # Initialize log file
    echo "Crowdfunding Farmer Contract Deployment Log" > "$DEPLOYMENT_LOG"
    echo "Started at: $(date -u +"%Y-%m-%d %H:%M:%S UTC")" >> "$DEPLOYMENT_LOG"
    echo "Network: $NETWORK" >> "$DEPLOYMENT_LOG"
    echo "Profile: $PROFILE" >> "$DEPLOYMENT_LOG"
    echo "========================================" >> "$DEPLOYMENT_LOG"
}

# Build the contract
build_contract() {
    print_header "BUILDING CONTRACT"

    cd "$CONTRACT_DIR"

    print_status "Building contract with build profile: release"
    log_with_timestamp "Starting contract build"

    # Build using stellar contract build
    if ! stellar contract build --profile release 2>&1 | tee -a "$DEPLOYMENT_LOG"; then
        print_error "Contract build failed"
        log_with_timestamp "Contract build failed"
        exit 1
    fi

    # Verify WASM file exists (check from project root since we cd'd to contract dir)
    if [[ ! -f "$WASM_PATH" ]]; then
        # Try alternate path for older CLI versions
        ALT_WASM_PATH="${CONTRACT_DIR}/target/wasm32-unknown-unknown/release/crowdfunding_farmer_contract.wasm"
        if [[ -f "$ALT_WASM_PATH" ]]; then
            WASM_PATH="$ALT_WASM_PATH"
            print_status "Using WASM from alternate location: $WASM_PATH"
        else
            # Try another common location (workspace builds put WASM in project root target)
            ALT_WASM_PATH2="${PROJECT_ROOT}/target/wasm32-unknown-unknown/release/crowdfunding_farmer_contract.wasm"
            if [[ -f "$ALT_WASM_PATH2" ]]; then
                WASM_PATH="$ALT_WASM_PATH2"
                print_status "Using WASM from workspace location: $WASM_PATH"
            else
                print_error "WASM file not found at expected paths"
                print_status "Searched locations:"
                echo "  - ${PROJECT_ROOT}/target/wasm32v1-none/release/crowdfunding_farmer_contract.wasm"
                echo "  - ${PROJECT_ROOT}/target/wasm32-unknown-unknown/release/crowdfunding_farmer_contract.wasm"
                echo "  - ${CONTRACT_DIR}/target/wasm32-unknown-unknown/release/crowdfunding_farmer_contract.wasm"
                print_status "Available WASM files:"
                find "$PROJECT_ROOT/target" -name "*.wasm" 2>/dev/null | head -10 | tee -a "$DEPLOYMENT_LOG" || true
                exit 1
            fi
        fi
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
    if ! upload_output=$(stellar contract upload --source-account "$PROFILE" --network "$NETWORK" --wasm "$WASM_PATH" 2>&1); then
        print_error "Contract upload failed"
        echo "$upload_output" | tee -a "$DEPLOYMENT_LOG"
        log_with_timestamp "Contract upload failed"
        exit 1
    fi

    # Parse WASM hash from output (plain text - 64 character hex string)
    WASM_HASH=$(echo "$upload_output" | grep -E '^[a-f0-9]{64}$' | head -1)

    # Validate it's a proper 64-character hex string
    if [[ -z "$WASM_HASH" || ! "$WASM_HASH" =~ ^[a-f0-9]{64}$ ]]; then
        print_error "Failed to extract valid WASM hash from upload output"
        echo "Upload output: $upload_output" | tee -a "$DEPLOYMENT_LOG"
        echo "Extracted value: '$WASM_HASH'" | tee -a "$DEPLOYMENT_LOG"
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
    if ! deploy_output=$(stellar contract deploy --source-account "$PROFILE" --network "$NETWORK" --wasm-hash "$WASM_HASH" 2>&1); then
        print_error "Contract deployment failed"
        echo "$deploy_output" | tee -a "$DEPLOYMENT_LOG"
        log_with_timestamp "Contract deployment failed"
        exit 1
    fi

    # Parse contract ID from output (plain text - starts with C, 56 characters)
    CONTRACT_ID=$(echo "$deploy_output" | grep -E '^C[A-Z0-9]{55}$' | tail -1)

    # Validate it's a proper Stellar contract ID (starts with C, 56 characters)
    if [[ -z "$CONTRACT_ID" || ! "$CONTRACT_ID" =~ ^C[A-Z0-9]{55}$ ]]; then
        print_error "Failed to extract valid contract ID from deployment output"
        echo "Deployment output: $deploy_output" | tee -a "$DEPLOYMENT_LOG"
        echo "Extracted value: '$CONTRACT_ID'" | tee -a "$DEPLOYMENT_LOG"
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
Crowdfunding Farmer Contract Deployment Summary
===============================================
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
    echo "  4. Create and fund campaigns"
    echo ""

    # Network-specific instructions
    if [[ "$NETWORK" == "testnet" ]]; then
        echo -e "${BLUE}Testnet Explorer:${NC} https://testnet.stellar.org/explorer"
        echo -e "${BLUE}Contract URL:${NC} https://testnet.stellar.org/explorer/contract/$CONTRACT_ID"
    else
        echo -e "${BLUE}Mainnet Explorer:${NC} https://stellar.org/explorer"
        echo -e "${BLUE}Contract URL:${NC} https://stellar.org/explorer/contract/$CONTRACT_ID"
    fi

    log_with_timestamp "Deployment process completed successfully"
}

# Print usage information
print_usage() {
    echo "Usage: ./deploy_crowdfunding_farmer.zsh [network] [identity]"
    echo ""
    echo "Arguments:"
    echo "  network    - Target network (testnet or mainnet)"
    echo "  identity   - Stellar identity/key to use (optional, defaults to 'default')"
    echo ""
    echo "Examples:"
    echo "  ./deploy_crowdfunding_farmer.zsh testnet                  # Deploy to testnet with default identity"
    echo "  ./deploy_crowdfunding_farmer.zsh testnet alice            # Deploy to testnet with 'alice' identity"
    echo "  ./deploy_crowdfunding_farmer.zsh mainnet production       # Deploy to mainnet with production identity"
    echo ""
    echo "Prerequisites:"
    echo "  - Stellar CLI installed (cargo install stellar-cli)"
    echo "  - Valid Stellar identity configured (stellar keys generate <name> --network <network>)"
    echo "  - Optional: jq for JSON output parsing"
    echo ""
    echo "The script will:"
    echo "  1. Build the crowdfunding farmer contract"
    echo "  2. Upload the WASM to the specified network"
    echo "  3. Deploy the contract and capture the contract ID"
    echo "  4. Save deployment logs and results"
    echo ""
    echo "Contract Features:"
    echo "  - Create and manage farmer crowdfunding campaigns"
    echo "  - Handle contributions from supporters"
    echo "  - Distribute rewards to contributors"
    echo "  - Track campaign progress and status"
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