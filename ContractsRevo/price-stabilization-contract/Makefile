# Makefile for Price Stabilization Contract

SHELL := bash
.ONESHELL:
SHELLFLAGS := -eu -o pipefail -c

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
ifeq ($(strip $(ADMIN_SECRET)),)
  $(error ADMIN_SECRET environment variable is not set)
endif

deploy-testnet:
	@echo "Deploying to Stellar testnet..."
	stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/price_stabilization_contract.wasm \
		--network testnet \
		--source $(ADMIN_SECRET)

# Deploy to Stellar mainnet (use with caution)
ifeq ($(strip $(ADMIN_SECRET)),)
  $(error ADMIN_SECRET environment variable is not set)
endif

deploy-mainnet:
	@echo "Deploying to Stellar mainnet..."
	@read -p "Are you sure you want to deploy to MAINNET? (yes/no): " CONF; \
	if [ "$$CONF" != "yes" ]; then \
	  echo "Aborting mainnet deployment."; \
	  exit 1; \
	fi; \
	stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/price_stabilization_contract.wasm \
		--network mainnet \
		--source $(ADMIN_SECRET)

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open