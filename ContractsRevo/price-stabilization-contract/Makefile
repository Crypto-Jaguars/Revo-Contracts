# Makefile for Price Stabilization Contract

.PHONY: all build test clean deploy

all: build test

build:
	@echo "Building Price Stabilization Contract..."
	cargo build --release
	stellar contract build

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Deploy to Stellar testnet
deploy-testnet:
	@echo "Deploying to Stellar testnet..."
	stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/price_stabilization_contract.wasm \
		--network testnet \
		--source $(ADMIN_SECRET)

# Deploy to Stellar mainnet (use with caution)
deploy-mainnet:
	@echo "Deploying to Stellar mainnet..."
	@echo "WARNING: You are about to deploy to mainnet. Press Ctrl+C to cancel or Enter to continue."
	@read
	stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/price_stabilization_contract.wasm \
		--network public \
		--source $(ADMIN_SECRET)

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open