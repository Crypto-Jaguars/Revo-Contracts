#!/bin/bash

# Microlending Contract Deployment Script
# This script automates the deployment process for the microlending contract

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command -v soroban &> /dev/null; then
        print_error "Soroban CLI is not installed. Please install it first:"
        echo "cargo install soroban-cli"
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Build the contract
build_contract() {
    print_status "Building the microlending contract..."
    
    # Build with Cargo
    cargo build --target wasm32-unknown-unknown --release
    
    # Build with Soroban CLI
    soroban contract build
    
    print_success "Contract built successfully"
}

# Deploy to testnet
deploy_testnet() {
    local source_account="$1"
    
    if [ -z "$source_account" ]; then
        print_error "Source account is required for deployment"
        echo "Usage: $0 testnet <source-account>"
        exit 1
    fi
    
    print_status "Deploying to Stellar testnet..."
    print_warning "Make sure your account has sufficient XLM for deployment fees"
    
    # Upload the contract
    print_status "Uploading contract..."
    soroban contract upload \
        --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
        --network testnet \
        --source "$source_account"
    
    # Deploy the contract
    print_status "Deploying contract..."
    local contract_id
    contract_id=$(soroban contract deploy \
        --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
        --network testnet \
        --source "$source_account")
    
    print_success "Contract deployed successfully!"
    echo "Contract ID: $contract_id"
    
    # Update DEPLOYMENTS.md
    update_deployments_file "$contract_id" "testnet"
    
    return 0
}

# Deploy to mainnet
deploy_mainnet() {
    local source_account="$1"
    
    if [ -z "$source_account" ]; then
        print_error "Source account is required for deployment"
        echo "Usage: $0 mainnet <source-account>"
        exit 1
    fi
    
    print_warning "You are about to deploy to MAINNET!"
    print_warning "This will cost real XLM and cannot be undone."
    read -p "Are you sure you want to continue? (yes/no): " confirm
    
    if [ "$confirm" != "yes" ]; then
        print_status "Deployment cancelled"
        exit 0
    fi
    
    print_status "Deploying to Stellar mainnet..."
    
    # Upload the contract
    print_status "Uploading contract..."
    soroban contract upload \
        --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
        --network mainnet \
        --source "$source_account"
    
    # Deploy the contract
    print_status "Deploying contract..."
    local contract_id
    contract_id=$(soroban contract deploy \
        --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
        --network mainnet \
        --source "$source_account")
    
    print_success "Contract deployed successfully to mainnet!"
    echo "Contract ID: $contract_id"
    
    # Update DEPLOYMENTS.md
    update_deployments_file "$contract_id" "mainnet"
    
    return 0
}

# Update DEPLOYMENTS.md file
update_deployments_file() {
    local contract_id="$1"
    local network="$2"
    local date=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
    
    print_status "Updating DEPLOYMENTS.md..."
    
    # Create a temporary file with updated content
    local temp_file=$(mktemp)
    
    # Update the deployments file
    if [ "$network" = "testnet" ]; then
        sed "s/| TBD | testnet | TBD | pending |/| $contract_id | testnet | $date | deployed |/" DEPLOYMENTS.md > "$temp_file"
    else
        sed "s/| TBD | mainnet | TBD | pending |/| $contract_id | mainnet | $date | deployed |/" DEPLOYMENTS.md > "$temp_file"
    fi
    
    mv "$temp_file" DEPLOYMENTS.md
    print_success "DEPLOYMENTS.md updated with contract ID: $contract_id"
}

# Test the deployed contract
test_contract() {
    local contract_id="$1"
    local network="$2"
    local source_account="$3"
    
    if [ -z "$contract_id" ] || [ -z "$network" ] || [ -z "$source_account" ]; then
        print_error "Contract ID, network, and source account are required for testing"
        return 1
    fi
    
    print_status "Testing deployed contract..."
    
    # Test basic contract functionality
    print_status "Testing contract initialization..."
    
    # Note: This is a placeholder - actual testing would require a token address
    print_warning "Manual testing required:"
    echo "1. Initialize the contract with a token address:"
    echo "   soroban contract invoke --id $contract_id --source $source_account --network $network -- initialize --token_address <TOKEN_ADDRESS>"
    echo ""
    echo "2. Create a test loan request:"
    echo "   soroban contract invoke --id $contract_id --source $source_account --network $network -- create_loan_request --borrower <BORROWER_ADDRESS> --amount 1000 --purpose 'Test loan' --duration_days 30 --interest_rate 1000 --collateral '{\"asset_type\": \"Test\", \"estimated_value\": 1500, \"verification_data\": \"0x0000000000000000000000000000000000000000000000000000000000000000\"}'"
}

# Main function
main() {
    local command="$1"
    local source_account="$2"
    
    case "$command" in
        "build")
            check_prerequisites
            build_contract
            ;;
        "testnet")
            check_prerequisites
            build_contract
            deploy_testnet "$source_account"
            ;;
        "mainnet")
            check_prerequisites
            build_contract
            deploy_mainnet "$source_account"
            ;;
        "test")
            local contract_id="$2"
            local network="$3"
            local source_account="$4"
            test_contract "$contract_id" "$network" "$source_account"
            ;;
        *)
            echo "Usage: $0 {build|testnet|mainnet|test} [source-account] [contract-id] [network]"
            echo ""
            echo "Commands:"
            echo "  build                    - Build the contract"
            echo "  testnet <source-account> - Deploy to testnet"
            echo "  mainnet <source-account> - Deploy to mainnet"
            echo "  test <contract-id> <network> <source-account> - Test deployed contract"
            echo ""
            echo "Examples:"
            echo "  $0 build"
            echo "  $0 testnet GBZXN7PIRZGNWCXXFYU7KYWXX4BXZUYHZO5QUEMKRHLUVLYN53WVFG3E"
            echo "  $0 test CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN testnet GBZXN7PIRZGNWCXXFYU7KYWXX4BXZUYHZO5QUEMKRHLUVLYN53WVFG3E"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
