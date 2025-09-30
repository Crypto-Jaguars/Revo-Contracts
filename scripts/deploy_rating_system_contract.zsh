#!/bin/zsh
# Usage: ./deploy_rating_system_contract.zsh [network] [profile] [source_account]
# Example: ./deploy_rating_system_contract.zsh testnet release SCL4L2HOILNHPQ4VTFV3AFRZEYFNPLFVSWKVDD5W4ZUUKRRM3TCFC6KH

set -e

NETWORK=$1
PROFILE=$2
SOURCE_ACCOUNT=$3

if [[ -z "$NETWORK" || -z "$PROFILE" || -z "$SOURCE_ACCOUNT" ]]; then
  echo "Usage: $0 [network: testnet|mainnet] [profile] [source_account]"
  exit 1
fi

CONTRACT_DIR="ContractsRevo/rating-system-contract"
WASM_PATH="$CONTRACT_DIR/target/wasm32v1-none/release/rating_system_contract.wasm"
LOG_FILE="scripts/deploy_rating_system_contract.log"

echo "Building contract..."
cd $CONTRACT_DIR
stellar contract build --profile $PROFILE

echo "Uploading contract WASM to $NETWORK..."
UPLOAD_OUTPUT=$(stellar contract upload --network $NETWORK --source-account $SOURCE_ACCOUNT --wasm $WASM_PATH 2>&1)
if [[ $? -ne 0 ]]; then
  echo "Error: Upload command failed. Output:"
  echo "$UPLOAD_OUTPUT"
  exit 2
fi
WASM_HASH=$(echo "$UPLOAD_OUTPUT" | grep -Eo '[a-f0-9]{64}' | head -n 1)

if [[ -z "$WASM_HASH" || "$WASM_HASH" == "null" ]]; then
  echo "Error: Failed to upload WASM or parse wasm_hash. Output:"
  echo "$UPLOAD_OUTPUT"
  exit 2
fi

echo "WASM Hash: $WASM_HASH"

echo "Deploying contract to $NETWORK..."
DEPLOY_OUTPUT=$(stellar contract deploy --network $NETWORK --source-account $SOURCE_ACCOUNT --wasm-hash $WASM_HASH 2>&1)
if [[ $? -ne 0 ]]; then
  echo "Error: Deploy command failed. Output:"
  echo "$DEPLOY_OUTPUT"
  exit 3
fi
CONTRACT_ID=$(echo "$DEPLOY_OUTPUT" | grep -Eo '[A-Z0-9]{56}' | head -n 1)

if [[ -z "$CONTRACT_ID" || "$CONTRACT_ID" == "null" ]]; then
  echo "Error: Failed to deploy contract or parse contract_id. Output:"
  echo "$DEPLOY_OUTPUT"
  exit 3
fi

echo "Contract ID: $CONTRACT_ID"

# Save deployment results to log file
cd - > /dev/null
{
  echo "[$(date)] Network: $NETWORK, Profile: $PROFILE"
  echo "WASM Hash: $WASM_HASH"
  echo "Contract ID: $CONTRACT_ID"
  echo ""
} >> $LOG_FILE

echo "Deployment complete. Details saved to $LOG_FILE."
