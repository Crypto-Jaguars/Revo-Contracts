#!/bin/zsh

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Directory containing contracts
CONTRACTS_DIR="ContractsRevo"

echo "${GREEN}Starting deployment script...${NC}"

# Check for required environment variable
# if [ -z "$STELLAR_SECRET_KEY" ]; then
#     echo "${RED}Error: STELLAR_SECRET_KEY environment variable is not set${NC}"
#     echo "Please set it using: export STELLAR_SECRET_KEY=your_secret_key"
#     exit 1
# fi

# Check if contracts directory exists
if [ ! -d "$CONTRACTS_DIR" ]; then
    echo "${RED}Error: $CONTRACTS_DIR directory not found${NC}"
    exit 1
fi


# Function to build and deploy a contract
deploy_contract() {
    local contract=$1
    echo "${GREEN}Processing $contract...${NC}"
    

    # Create a log file for this contract
    local log_file="logs/${contract}_$(date +%Y%m%d_%H%M%S).log"
    mkdir -p logs
    
    {
        # Build contract
        echo "Building..."
        if ! soroban contract build --package $contract; then
            echo "${RED}Failed to build contract ${contract}${NC}"
            echo "Build failed for ${contract}" >> failed_contracts.txt
            return 1
        fi
        
        # Get contract ID from built WASM
        sanitized_contract_name=$(echo "$contract" | tr '-' '_')
        local wasm_path="target/wasm32v1-none/release/$sanitized_contract_name.wasm"
        
        # Check if WASM file exists
        if [ ! -f "$wasm_path" ]; then
            echo "${RED}WASM file not found at ${wasm_path}${NC}"
            echo "WASM not found for ${contract}" >> failed_contracts.txt
            return 1
        fi
        
        # Deploy to testnet
        echo "Deploying to testnet..."
        local contract_id
        if ! contract_id=$(soroban contract deploy \
            --wasm $wasm_path \
            --network testnet \
            --source-account $STELLAR_SECRET_KEY); then
            echo "${RED}Failed to deploy contract ${contract}${NC}"
            echo "Deploy failed for ${contract}" >> failed_contracts.txt
            return 1
        fi
        
        echo "${GREEN}Contract $contract deployed with ID: $contract_id${NC}"
        echo "$contract: $contract_id" >> deployed_contracts.txt
        return 0
        
    } 2>&1 | tee "$log_file"
}

# Initialize log files
echo "Deployment Log - $(date)" > deployed_contracts.txt
echo "Failed Contracts - $(date)" > failed_contracts.txt


total_count=0

# Process each contract
for contract in $CONTRACTS_DIR/*/; do
    if [ -f "$contract/Cargo.toml" ]; then
        contract_name=$(basename $contract)
        ((total_count++)) || true
        deploy_contract $contract_name
    fi
done

echo "${GREEN}✅ Deployment complete! Check deployed_contracts.txt for contract IDs${NC}"

# Print summary
echo "\n${GREEN}Deployment Summary:${NC}"
echo "✅ Total contracts: $total_count"
echo "See deployed_contracts.txt for successful deployments"
echo "See failed_contracts.txt for failed deployments"
echo "Detailed logs available in the logs directory"