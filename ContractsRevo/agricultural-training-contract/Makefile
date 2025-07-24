# Makefile for the Soroban Agricultural Training Smart Contract

.PHONY: all build test clean deploy init

# ===================================================================
#                          VARIABLES
# ===================================================================
# Set your identity name from `soroban config identity ls`
ADMIN_ACCOUNT :=
INSTRUCTOR_ACCOUNT :=
FARMER_ACCOUNT :=

# Set the contract IDs after deploying the dependencies and this contract
CERTIFICATE_CONTRACT_ID :=
LOYALTY_CONTRACT_ID :=
TRAINING_CONTRACT_ID :=

# This is a unique ID for a specific program within the loyalty contract
LOYALTY_PROGRAM_ID :=

CONTRACT_NAME := agricultural_training_contract
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
# Usage: make deploy ADMIN_ACCOUNT=<your-admin-name>
deploy: build
	@echo "Deploying contract to $(NETWORK)..."
	@soroban contract deploy \
		--wasm $(WASM_PATH) \
		--source $(ADMIN_ACCOUNT) \
		--network $(NETWORK)

# Initialize the deployed contract
# Usage: make init ADMIN_ACCOUNT=<admin> TRAINING_CONTRACT_ID=<id> CERTIFICATE_CONTRACT_ID=<id> LOYALTY_CONTRACT_ID=<id> LOYALTY_PROGRAM_ID=<id>
init:
	@echo "Initializing contract on $(NETWORK)..."
	@soroban contract invoke \
		--id $(TRAINING_CONTRACT_ID) \
		--source $(ADMIN_ACCOUNT) \
		--network $(NETWORK) -- \
		initialize \
		--admin "$$(soroban config identity address $(ADMIN_ACCOUNT))" \
		--certificate_contract_id "$(CERTIFICATE_CONTRACT_ID)" \
		--loyalty_token_id "$(LOYALTY_CONTRACT_ID)" \
		--loyalty_program_id "$(LOYALTY_PROGRAM_ID)"

# ===================================================================
#                    EXAMPLE INVOCATIONS
# ===================================================================

# Create a new training program
# Usage: make create-program INSTRUCTOR_ACCOUNT=<instructor> TRAINING_CONTRACT_ID=<id>
create-program:
	@echo "Creating a new training program..."
	@soroban contract invoke \
		--id $(TRAINING_CONTRACT_ID) \
		--source $(INSTRUCTOR_ACCOUNT) \
		--network $(NETWORK) -- \
		create_training_program \
		--instructor "$$(soroban config identity address $(INSTRUCTOR_ACCOUNT))" \
		--title "Sustainable Soil Management" \
		--description "A course on modern soil health techniques." \
		--duration_hours 40 \
		--materials_hash '{"__unstable_bytes_encoded_for_testing":"...put_ipfs_hash_here..."}'

# Enroll a farmer in a program
# Usage: make enroll-farmer FARMER_ACCOUNT=<farmer> TRAINING_CONTRACT_ID=<id> PROGRAM_ID=<program-id>
enroll-farmer:
	@echo "Enrolling farmer..."
	@soroban contract invoke \
		--id $(TRAINING_CONTRACT_ID) \
		--source $(FARMER_ACCOUNT) \
		--network $(NETWORK) -- \
		enroll_farmer \
		--farmer "$$(soroban config identity address $(FARMER_ACCOUNT))" \
		--program_id "$(PROGRAM_ID)"

# Update a farmer's progress
# Usage: make update-progress INSTRUCTOR_ACCOUNT=<instructor> TRAINING_CONTRACT_ID=<id> PROGRAM_ID=<id> FARMER_ID=<farmer-pubkey>
update-progress:
	@echo "Updating farmer progress to 100%..."
	@soroban contract invoke \
		--id $(TRAINING_CONTRACT_ID) \
		--source $(INSTRUCTOR_ACCOUNT) \
		--network $(NETWORK) -- \
		update_progress \
		--instructor "$$(soroban config identity address $(INSTRUCTOR_ACCOUNT))" \
		--program_id "$(PROGRAM_ID)" \
		--farmer_id "$(FARMER_ID)" \
		--progress_percentage 100

# Issue a certificate to a completed farmer
# Usage: make issue-certificate INSTRUCTOR_ACCOUNT=<instructor> TRAINING_CONTRACT_ID=<id> PROGRAM_ID=<id> FARMER_ID=<farmer-pubkey>
issue-certificate:
	@echo "Issuing certificate and rewards..."
	@soroban contract invoke \
		--id $(TRAINING_CONTRACT_ID) \
		--source $(INSTRUCTOR_ACCOUNT) \
		--network $(NETWORK) -- \
		issue_certificate \
		--instructor "$$(soroban config identity address $(INSTRUCTOR_ACCOUNT))" \
		--program_id "$(PROGRAM_ID)" \
		--farmer_id "$(FARMER_ID)"

