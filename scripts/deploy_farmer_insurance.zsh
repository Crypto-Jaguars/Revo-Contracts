#!/bin/zsh

# Farmer Insurance Contract Deployment Script
# Usage: ./deploy_farmer_insurance.zsh [network] [profile] [source_identity]
# Example: ./deploy_farmer_insurance.zsh testnet default alice

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
CONTRACT_NAME="farmer-insurance-contract"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
CONTRACT_DIR="${REPO_ROOT}/ContractsRevo/farmer-insurance-contract"
WASM_NAME="farmer_insurance_contract"
WASM_PATH="${REPO_ROOT}/target/wasm32v1-none/release/${WASM_NAME}.wasm"
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

# Function to cleanup on exit
cleanup() {
    if [ $? -ne 0 ]; then
        print_error "Deployment failed. Check the logs at: $DEPLOYMENT_LOG"
    fi
}
trap cleanup EXIT

# Function to validate inputs
validate_inputs() {
    local network=$1
    local profile=$2
    local source_identity=$3
    
    if [[ "$network" != "testnet" && "$network" != "mainnet" ]]; then
        print_error "Invalid network. Use 'testnet' or 'mainnet'"
        exit 1
    fi
    
    if [[ -z "$profile" ]]; then
        print_error "Profile is required"
        exit 1
    fi
    
    if [[ -z "$source_identity" ]]; then
        print_error "Source identity is required"
        exit 1
    fi
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if stellar CLI is installed
    if ! command -v stellar &> /dev/null; then
        print_error "Stellar CLI is not installed or not in PATH"
        exit 1
    fi
    
    # Check if jq is installed for JSON parsing
    if ! command -v jq &> /dev/null; then
        print_warning "jq is not installed. JSON parsing may not work properly"
    fi
    
    # Check if contract directory exists
    if [[ ! -d "$CONTRACT_DIR" ]]; then
        print_error "Contract directory not found: $CONTRACT_DIR"
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to create log directory
setup_logging() {
    if [[ ! -d "$LOG_DIR" ]]; then
        mkdir -p "$LOG_DIR"
        print_status "Created log directory: $LOG_DIR"
    fi
    
    # Initialize log file
    log_with_timestamp "Starting deployment of $CONTRACT_NAME"
    log_with_timestamp "Network: $NETWORK"
    log_with_timestamp "Profile: $PROFILE"
    log_with_timestamp "Source Identity: $SOURCE_IDENTITY"
    log_with_timestamp "Contract Directory: $CONTRACT_DIR"
    log_with_timestamp "WASM Path: $WASM_PATH"
}

# Function to build the contract
build_contract() {
    print_header "Building Contract"
    
    cd "$CONTRACT_DIR"
    
    print_status "Building $CONTRACT_NAME with profile: $PROFILE"
    log_with_timestamp "Build command: stellar contract build --profile $PROFILE"
    
    if stellar contract build --profile "$PROFILE" 2>&1 | tee -a "$DEPLOYMENT_LOG"; then
        print_success "Contract built successfully"
        
        # Verify WASM file exists
        if [[ -f "$WASM_PATH" ]]; then
            WASM_SIZE=$(wc -c < "$WASM_PATH")
            print_status "WASM file created: $WASM_PATH (${WASM_SIZE} bytes)"
            log_with_timestamp "WASM file size: ${WASM_SIZE} bytes"
        else
            print_error "WASM file not found after build: $WASM_PATH"
            exit 1
        fi
    else
        print_error "Failed to build contract"
        exit 1
    fi
}

# Function to upload the contract
upload_contract() {
    print_header "Uploading Contract"
    
    print_status "Uploading WASM to $NETWORK network..."
    log_with_timestamp "Upload command: stellar contract upload --source $SOURCE_IDENTITY --network $NETWORK --wasm $WASM_PATH"
    
    # Execute upload command and capture output
    UPLOAD_OUTPUT=$(stellar contract upload --source "$SOURCE_IDENTITY" --network "$NETWORK" --wasm "$WASM_PATH" 2>&1)
    UPLOAD_EXIT_CODE=$?
    
    # Log the full output
    echo "$UPLOAD_OUTPUT" >> "$DEPLOYMENT_LOG"
    
    if [ $UPLOAD_EXIT_CODE -eq 0 ]; then
        # Extract WASM hash from output (stellar CLI outputs the hash directly)
        WASM_HASH=$(echo "$UPLOAD_OUTPUT" | tail -n 1 | tr -d '\n')
        
        if [[ -n "$WASM_HASH" && ${#WASM_HASH} -eq 64 ]]; then
            print_success "Contract uploaded successfully"
            print_status "WASM Hash: $WASM_HASH"
            log_with_timestamp "WASM Hash: $WASM_HASH"
        else
            print_error "Failed to extract WASM hash from upload output"
            print_error "Upload output: $UPLOAD_OUTPUT"
            exit 1
        fi
    else
        print_error "Failed to upload contract"
        print_error "Upload output: $UPLOAD_OUTPUT"
        exit 1
    fi
}

# Function to deploy the contract
deploy_contract() {
    print_header "Deploying Contract"
    
    print_status "Deploying contract to $NETWORK network..."
    log_with_timestamp "Deploy command: stellar contract deploy --source $SOURCE_IDENTITY --network $NETWORK --wasm $WASM_PATH"
    
    # Execute deploy command and capture output
    DEPLOY_OUTPUT=$(stellar contract deploy --source "$SOURCE_IDENTITY" --network "$NETWORK" --wasm "$WASM_PATH" 2>&1)
    DEPLOY_EXIT_CODE=$?
    
    # Log the full output
    echo "$DEPLOY_OUTPUT" >> "$DEPLOYMENT_LOG"
    
    if [ $DEPLOY_EXIT_CODE -eq 0 ]; then
        # Extract contract ID from output (stellar CLI outputs the contract ID directly)
        CONTRACT_ID=$(echo "$DEPLOY_OUTPUT" | tail -n 1 | tr -d '\n')
        
        if [[ -n "$CONTRACT_ID" && ${#CONTRACT_ID} -eq 56 ]]; then
            print_success "Contract deployed successfully"
            print_status "Contract ID: $CONTRACT_ID"
            log_with_timestamp "Contract ID: $CONTRACT_ID"
        else
            print_error "Failed to extract Contract ID from deploy output"
            print_error "Deploy output: $DEPLOY_OUTPUT"
            exit 1
        fi
    else
        print_error "Failed to deploy contract"
        print_error "Deploy output: $DEPLOY_OUTPUT"
        exit 1
    fi
}

# Function to save deployment summary
save_deployment_summary() {
    print_header "Deployment Summary"
    
    SUMMARY_FILE="${LOG_DIR}/deployment_summary_$(date +%Y%m%d_%H%M%S).json"
    
    cat > "$SUMMARY_FILE" << EOF
{
  "contract_name": "$CONTRACT_NAME",
  "network": "$NETWORK",
  "profile": "$PROFILE",
  "source_identity": "$SOURCE_IDENTITY",
  "deployment_timestamp": "$(date -u +"%Y-%m-%d %H:%M:%S UTC")",
  "wasm_hash": "$WASM_HASH",
  "contract_id": "$CONTRACT_ID",
  "wasm_path": "$WASM_PATH",
  "wasm_size_bytes": $(wc -c < "$WASM_PATH"),
  "log_file": "$DEPLOYMENT_LOG"
}
EOF
    
    print_success "Deployment summary saved to: $SUMMARY_FILE"
    
    # Display summary
    echo ""
    print_status "=== DEPLOYMENT RESULTS ==="
    print_status "Contract Name: $CONTRACT_NAME"
    print_status "Network: $NETWORK"
    print_status "Profile: $PROFILE"
    print_status "Source Identity: $SOURCE_IDENTITY"
    print_status "WASM Hash: $WASM_HASH"
    print_status "Contract ID: $CONTRACT_ID"
    print_status "Deployment Time: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    print_status "Log File: $DEPLOYMENT_LOG"
    print_status "Summary File: $SUMMARY_FILE"
    echo ""
    
    # Save to main deployment log
    log_with_timestamp "=== DEPLOYMENT SUMMARY ==="
    log_with_timestamp "WASM Hash: $WASM_HASH"
    log_with_timestamp "Contract ID: $CONTRACT_ID"
    log_with_timestamp "Summary saved to: $SUMMARY_FILE"
}

# Function to verify deployment
verify_deployment() {
    print_header "Verifying Deployment"
    
    print_status "Verifying contract deployment..."
    
    # Try to get contract info
    if stellar contract invoke --id "$CONTRACT_ID" --source "$SOURCE_IDENTITY" --network "$NETWORK" -- --help >/dev/null 2>&1; then
        print_success "Contract verification successful - contract is accessible"
        log_with_timestamp "Contract verification: SUCCESS"
    else
        print_warning "Contract verification failed - contract may not be immediately available"
        log_with_timestamp "Contract verification: FAILED"
    fi
}

# Function to display usage
usage() {
    echo "Usage: $0 [network] [profile] [source_identity]"
    echo ""
    echo "Parameters:"
    echo "  network         : testnet or mainnet"
    echo "  profile         : build profile (default, release, etc.)"
    echo "  source_identity : stellar identity name for deployment"
    echo ""
    echo "Examples:"
    echo "  $0 testnet default alice"
    echo "  $0 mainnet release production_key"
    echo ""
    exit 1
}

# Main execution
main() {
    # Parse command line arguments
    NETWORK=${1:-"testnet"}
    PROFILE=${2:-"default"}
    SOURCE_IDENTITY=${3:-"alice"}
    
    # Show usage if help requested
    if [[ "$1" == "-h" || "$1" == "--help" ]]; then
        usage
    fi
    
    # Validate inputs
    validate_inputs "$NETWORK" "$PROFILE" "$SOURCE_IDENTITY"
    
    print_header "Farmer Insurance Contract Deployment"
    print_status "Starting deployment process..."
    
    # Check prerequisites
    check_prerequisites
    
    # Setup logging
    setup_logging
    
    # Build contract
    build_contract
    
    # Upload contract
    upload_contract
    
    # Deploy contract
    deploy_contract
    
    # Save deployment summary
    save_deployment_summary
    
    # Verify deployment
    verify_deployment
    
    print_success "Deployment completed successfully!"
    log_with_timestamp "Deployment process completed successfully"
}

# Run main function with all arguments
main "$@"