#!/bin/zsh

# Colors for beautiful output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Function to print beautiful headers
print_header() {
    echo -e "\n${BOLD}${PURPLE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${PURPLE}║${NC} ${WHITE}$1${NC} ${BOLD}${PURPLE}║${NC}"
    echo -e "${BOLD}${PURPLE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}\n"
}

# Function to print success messages
log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print info messages
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to print warning messages
log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to print error messages
log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to print step messages
log_step() {
    echo -e "\n${CYAN}🔄 $1${NC}"
}

# Function to show usage
show_usage() {
    echo -e "${BOLD}${WHITE}Usage:${NC}"
    echo -e "  ${CYAN}./scripts/deploy_loyalty_token_contract.zsh [network] [profile]${NC}"
    echo -e "\n${BOLD}${WHITE}Parameters:${NC}"
    echo -e "  ${YELLOW}network${NC}    - Stellar network (testnet, mainnet, futurenet)"
    echo -e "  ${YELLOW}profile${NC}    - Stellar profile name"
    echo -e "\n${BOLD}${WHITE}Examples:${NC}"
    echo -e "  ${CYAN}./scripts/deploy_loyalty_token_contract.zsh testnet kennyv4${NC}"
    echo -e "  ${CYAN}./scripts/deploy_loyalty_token_contract.zsh mainnet production${NC}"
}

# Check if parameters are provided
if [ $# -ne 2 ]; then
    print_header "Loyalty Token Contract Deployment"
    log_error "Invalid number of parameters!"
    show_usage
    exit 1
fi

NETWORK=$1
PROFILE=$2

# Validate network parameter
if [[ "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" && "$NETWORK" != "futurenet" ]]; then
    log_error "Invalid network: $NETWORK"
    log_info "Valid networks: testnet, mainnet, futurenet"
    exit 1
fi

print_header "Loyalty Token Contract Deployment"
log_info "Network: ${BOLD}$NETWORK${NC}"
log_info "Profile: ${BOLD}$PROFILE${NC}"

# Check if Rust target is installed
log_step "Checking Rust target..."
if ! rustup target list | grep -q "wasm32v1-none (installed)"; then
    log_warning "wasm32v1-none target not found, installing..."
    rustup target add wasm32v1-none
    if [ $? -ne 0 ]; then
        log_error "Failed to add wasm32v1-none target"
        exit 1
    fi
    log_success "wasm32v1-none target installed successfully"
else
    log_success "wasm32v1-none target is already installed"
fi

# Build the contracts
log_step "Building contracts..."
cargo build --target wasm32v1-none --release
if [ $? -ne 0 ]; then
    log_error "Build failed"
    exit 1
fi
log_success "Contracts built successfully"

# Loyalty Token Contract deployment
print_header "Deploying Loyalty Token Contract"

log_step "Uploading loyalty_token_contract to $NETWORK..."
loyalty_token_contract_upload_output=$(stellar contract upload \
  --network $NETWORK \
  --source $PROFILE \
  --wasm target/wasm32v1-none/release/loyalty_token_contract.wasm)

if [ $? -ne 0 ]; then
    log_error "Failed to upload contract"
    exit 1
fi

log_success "Contract uploaded successfully"
log_info "WASM Hash: ${BOLD}$loyalty_token_contract_upload_output${NC}"

log_step "Deploying loyalty_token_contract..."
loyalty_token_contract_deploy_output=$(stellar contract deploy \
  --wasm-hash $loyalty_token_contract_upload_output \
  --source $PROFILE \
  --network $NETWORK \
  --alias LoyaltyTokenContract)

if [ $? -ne 0 ]; then
    log_error "Failed to deploy contract"
    exit 1
fi

print_header "Deployment Complete"
log_success "Loyalty Token Contract deployed successfully!"
log_info "Contract ID: ${BOLD}$loyalty_token_contract_deploy_output${NC}"
log_info "Network: ${BOLD}$NETWORK${NC}"
log_info "Profile: ${BOLD}$PROFILE${NC}"

echo -e "\n${GREEN}🎉 Deployment completed successfully! 🎉${NC}"