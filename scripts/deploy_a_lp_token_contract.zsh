#!/bin/zsh

# a-lp-token-contract Deployment Script
# Usage: ./deploy_a_lp_token_contract.zsh [network] [profile]
# Example: ./deploy_a_lp_token_contract.zsh testnet release
#
# This script builds, uploads, and deploys the a-lp-token-contract to
# Stellar Testnet/Mainnet using the Stellar CLI.
#
# Requirements:
# - Stellar CLI: cargo install stellar-cli
# - Rust & Cargo: https://rustup.rs/
# - jq (for JSON parsing): brew install jq | apt-get install jq
# - A configured Stellar identity/profile for the chosen network

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Config
CONTRACT_NAME="a-lp-token-contract"
SCRIPT_DIR="${0:A:h}"
PROJECT_ROOT="${SCRIPT_DIR:h}"
CONTRACT_DIR="${PROJECT_ROOT}/ContractsRevo/a-lp-token-contract"

# The Stellar CLI builds to workspace root: target/wasm32v1-none/<profile>
# Crate name uses underscores in the artifact filename
ARTIFACT_NAME="a_lp_token_contract.wasm"

# Defaults
NETWORK_DEFAULT="testnet"
PROFILE_DEFAULT="release"

# Derived at runtime
NETWORK=""
PROFILE=""
WASM_DIR=""
WASM_PATH=""

LOG_DIR="${CONTRACT_DIR}/logs"
DEPLOYMENT_LOG="${LOG_DIR}/deployment_$(date +%Y%m%d_%H%M%S).log"

# Helpers
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

log_with_timestamp() {
  echo "[$(date -u +"%Y-%m-%d %H:%M:%S UTC")] $1" | tee -a "$DEPLOYMENT_LOG"
}

print_usage() {
  echo "Usage: ./deploy_a_lp_token_contract.zsh [network] [profile]"
  echo ""
  echo "Arguments:"
  echo "  network    - Target network (testnet or mainnet). Default: ${NETWORK_DEFAULT}"
  echo "  profile    - Build profile passed to 'stellar contract build --profile'. Default: ${PROFILE_DEFAULT}"
  echo ""
  echo "Examples:"
  echo "  ./deploy_a_lp_token_contract.zsh testnet release"
  echo "  ./deploy_a_lp_token_contract.zsh mainnet default"
  echo ""
  echo "The script will:"
  echo "  1. Build the contract (stellar contract build --profile <profile>)"
  echo "  2. Upload the built WASM and capture wasm_hash"
  echo "  3. Deploy the contract and capture contract_id"
  echo "  4. Save deployment logs and JSON results"
}

check_prerequisites() {
  print_header "PREREQUISITES"

  if ! command -v stellar >/dev/null 2>&1; then
    print_error "Stellar CLI is not installed. Install with: cargo install stellar-cli"
    exit 1
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    print_error "Cargo is not installed. Install Rust from https://rustup.rs/"
    exit 1
  fi

  if ! command -v jq >/dev/null 2>&1; then
    print_warning "jq is not installed. JSON parsing requires jq."
  fi

  print_success "Prerequisites OK"
}

validate_parameters() {
  local network="$1"
  local profile="$2"

  if [[ -z "$network" ]]; then
    network="$NETWORK_DEFAULT"
    print_warning "No network provided; defaulting to '${network}'"
  fi
  if [[ -z "$profile" ]]; then
    profile="$PROFILE_DEFAULT"
    print_warning "No profile provided; defaulting to '${profile}'"
  fi

  if [[ "$network" != "testnet" && "$network" != "mainnet" ]]; then
    print_error "Invalid network: $network. Must be 'testnet' or 'mainnet'"
    print_usage
    exit 1
  fi

  NETWORK="$network"
  PROFILE="$profile"

  WASM_DIR="${PROJECT_ROOT}/target/wasm32v1-none/${PROFILE}"
  WASM_PATH="${WASM_DIR}/${ARTIFACT_NAME}"

  print_success "Parameters validated: network=$NETWORK, profile=$PROFILE"
}

setup_logging() {
  mkdir -p "$LOG_DIR"
  : > "$DEPLOYMENT_LOG"
  print_status "Logging to: $DEPLOYMENT_LOG"
}

build_contract() {
  print_header "BUILD"
  log_with_timestamp "Building contract: ${CONTRACT_NAME} (profile=${PROFILE})"

  cd "$CONTRACT_DIR"
  if ! stellar contract build --profile "$PROFILE" 2>&1 | tee -a "$DEPLOYMENT_LOG"; then
    print_error "Contract build failed"
    exit 1
  fi

  # Verify WASM path
  if [[ ! -f "$WASM_PATH" ]]; then
    print_warning "Expected WASM not found at: $WASM_PATH"
    print_status "Searching for artifact '${ARTIFACT_NAME}' under '${PROJECT_ROOT}/target'"
    FOUND_PATH=$(find "$PROJECT_ROOT/target" -type f -name "$ARTIFACT_NAME" 2>/dev/null | head -n 1)
    if [[ -n "$FOUND_PATH" ]]; then
      WASM_PATH="$FOUND_PATH"
      print_status "Using discovered WASM: $WASM_PATH"
    else
      print_error "WASM artifact not found. Ensure build succeeded and correct profile is used."
      exit 1
    fi
  else
    print_success "WASM built: $WASM_PATH"
  fi
}

upload_contract() {
  print_header "UPLOAD"
  log_with_timestamp "Uploading WASM to $NETWORK"

  # Run upload and capture JSON
  local upload_output
  if ! upload_output=$(stellar contract upload --network "$NETWORK" --wasm "$WASM_PATH" --json 2>&1); then
    print_error "WASM upload failed"
    echo "$upload_output" | tee -a "$DEPLOYMENT_LOG"
    exit 1
  fi

  echo "$upload_output" | tee -a "$DEPLOYMENT_LOG"

  # Parse wasm_hash
  if command -v jq >/dev/null 2>&1; then
    WASM_HASH=$(echo "$upload_output" | jq -r '.wasm_hash')
  else
    WASM_HASH=$(echo "$upload_output" | sed -n 's/.*"wasm_hash"\s*:\s*"\([^"]*\)".*/\1/p')
  fi

  if [[ -z "$WASM_HASH" || "$WASM_HASH" == "null" ]]; then
    print_error "Failed to parse wasm_hash from upload output"
    exit 1
  fi

  print_success "WASM uploaded. Hash: $WASM_HASH"
}

deploy_contract() {
  print_header "DEPLOY"
  log_with_timestamp "Deploying contract on $NETWORK with wasm_hash=$WASM_HASH"

  local deploy_output
  if ! deploy_output=$(stellar contract deploy --network "$NETWORK" --wasm-hash "$WASM_HASH" --json 2>&1); then
    print_error "Contract deploy failed"
    echo "$deploy_output" | tee -a "$DEPLOYMENT_LOG"
    exit 1
  fi

  echo "$deploy_output" | tee -a "$DEPLOYMENT_LOG"

  # Parse contract_id
  if command -v jq >/dev/null 2>&1; then
    CONTRACT_ID=$(echo "$deploy_output" | jq -r '.contract_id')
  else
    CONTRACT_ID=$(echo "$deploy_output" | sed -n 's/.*"contract_id"\s*:\s*"\([^"]*\)".*/\1/p')
  fi

  if [[ -z "$CONTRACT_ID" || "$CONTRACT_ID" == "null" ]]; then
    print_error "Failed to parse contract_id from deploy output"
    exit 1
  fi

  print_success "Contract deployed. ID: $CONTRACT_ID"
}

save_deployment_results() {
  print_header "RESULTS"
  mkdir -p "$LOG_DIR"

  local results_json="${LOG_DIR}/deployment_results.json"
  local summary_txt="${LOG_DIR}/latest_deployment.txt"

  cat > "$results_json" <<JSON
{
  "contract": "${CONTRACT_NAME}",
  "network": "${NETWORK}",
  "profile": "${PROFILE}",
  "wasm_path": "${WASM_PATH}",
  "wasm_hash": "${WASM_HASH}",
  "contract_id": "${CONTRACT_ID}",
  "timestamp_utc": "$(date -u +"%Y-%m-%d %H:%M:%S")"
}
JSON

  echo "Contract: ${CONTRACT_NAME}" > "$summary_txt"
  echo "Network: ${NETWORK}" >> "$summary_txt"
  echo "Profile: ${PROFILE}" >> "$summary_txt"
  echo "WASM Hash: ${WASM_HASH}" >> "$summary_txt"
  echo "Contract ID: ${CONTRACT_ID}" >> "$summary_txt"
  echo "Log: ${DEPLOYMENT_LOG}" >> "$summary_txt"

  print_status "Saved: $results_json"
  print_status "Saved: $summary_txt"
}

display_results() {
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
  echo "  3. Save contract ID for client integration"
  echo ""
  if [[ "$NETWORK" == "testnet" ]]; then
    echo -e "${BLUE}Testnet Explorer:${NC} https://testnet.stellar.org/explorer"
  else
    echo -e "${BLUE}Mainnet Explorer:${NC} https://stellar.org/explorer"
  fi
}

cleanup() {
  local exit_code=$?
  if [[ $exit_code -ne 0 ]]; then
    print_error "Deployment failed with exit code: $exit_code"
  fi
  exit $exit_code
}

trap cleanup EXIT

main() {
  if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    print_usage
    exit 0
  fi

  setup_logging
  check_prerequisites
  validate_parameters "$1" "$2"

  build_contract
  upload_contract
  deploy_contract
  save_deployment_results
  display_results
}

main "$@"