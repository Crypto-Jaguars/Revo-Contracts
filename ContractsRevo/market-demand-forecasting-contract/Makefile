# Makefile for the Soroban Market Demand Forecasting Smart Contract

.PHONY: all build test clean deploy init set-oracle register-product generate-forecast get-recommendation

# ===================================================================
#                          VARIABLES
# ===================================================================
# Set your identity name from `soroban config identity ls`
SOURCE_ACCOUNT :=
# Set the contract ID after deploying
CONTRACT_ID :=
# Set the oracle's public key
ORACLE_ADDRESS :=

CONTRACT_NAME := market_demand_forecasting_contract
WASM_PATH := target/wasm32-unknown-unknown/release/$(CONTRACT_NAME).wasm
NETWORK := testnet

# ===================================================================
#                      CORE COMMANDS
# ===================================================================

# Default target
all: build

# Build the smart contract to WASM
build:
	@echo "Building the contract..."
	@soroban contract build

# Run tests
test:
	@echo "Running tests..."
	@cargo test -- --nocapture

# Clean the build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# ===================================================================
#                      DEPLOYMENT & SETUP
# ===================================================================

# Deploy the contract to the testnet
# Usage: make deploy SOURCE_ACCOUNT=<your-account-name>
deploy: build
	@echo "Deploying contract to $(NETWORK)..."
	@soroban contract deploy \
		--wasm $(WASM_PATH) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK)

# Initialize the deployed contract
# Usage: make init SOURCE_ACCOUNT=<your-account-name> CONTRACT_ID=<deployed-contract-id>
init:
	@echo "Initializing contract on $(NETWORK)..."
	@soroban contract invoke \
		--id $(CONTRACT_ID) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK) -- \
		initialize \
		--admin "$$(soroban config identity address $(SOURCE_ACCOUNT))"

# Set the trusted oracle address
# Usage: make set-oracle SOURCE_ACCOUNT=<admin> CONTRACT_ID=<id> ORACLE_ADDRESS=<oracle-pubkey>
set-oracle:
	@echo "Setting oracle address..."
	@soroban contract invoke \
		--id $(CONTRACT_ID) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK) -- \
		set_oracle \
		--admin "$$(soroban config identity address $(SOURCE_ACCOUNT))" \
		--oracle "$(ORACLE_ADDRESS)"

# ===================================================================
#                    EXAMPLE INVOCATIONS
# ===================================================================

# Register a new product
# Usage: make register-product SOURCE_ACCOUNT=<any-user> CONTRACT_ID=<id>
register-product:
	@echo "Registering a new product (Wheat)..."
	@soroban contract invoke \
		--id $(CONTRACT_ID) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK) -- \
		register_product \
		--name "Wheat" \
		--historical_demand '[1000, 1200, 1150]'

# Generate a forecast (called by the oracle)
# Usage: make generate-forecast SOURCE_ACCOUNT=<oracle-account> CONTRACT_ID=<id>
generate-forecast:
	@echo "Generating a new forecast..."
	@soroban contract invoke \
		--id $(CONTRACT_ID) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK) -- \
		generate_forecast \
		--oracle "$$(soroban config identity address $(SOURCE_ACCOUNT))" \
		--product_id '{"__unstable_bytes_encoded_for_testing":"...put_product_id_here..."}' \
		--region "North" \
		--predicted_demand 1500 \
		--data_hash '{"__unstable_bytes_encoded_for_testing":"...put_ipfs_hash_here..."}'

# Get a crop recommendation
# Usage: make get-recommendation CONTRACT_ID=<id>
get-recommendation:
	@echo "Getting crop recommendation for the 'North' region..."
	@soroban contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) -- \
		generate_recommendation \
		--region "North" \
		--time_window_days 7

