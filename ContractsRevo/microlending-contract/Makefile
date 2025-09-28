# Makefile for Microlending Contract

SHELL := bash
.ONESHELL:
SHELLFLAGS := -eu -o pipefail -c

.PHONY: all build test clean deploy-testnet deploy-mainnet docs

all: build test

build:
	@echo "Building Microlending Contract..."
	cargo build --target wasm32-unknown-unknown --release
	soroban contract build

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

deploy-testnet: build
	@echo "Deploying Microlending Contract to Stellar testnet..."
	@echo "Uploading contract..."
	soroban contract upload \
		--wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
		--network testnet \
		--source $(ADMIN_SECRET)
	@echo "Deploying contract..."
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
		--network testnet \
		--source $(ADMIN_SECRET)

# Deploy to Stellar mainnet (use with caution)
ifeq ($(strip $(ADMIN_SECRET)),)
  $(error ADMIN_SECRET environment variable is not set)
endif

deploy-mainnet: build
	@echo "Deploying Microlending Contract to Stellar mainnet..."
	@read -p "Are you sure you want to deploy to MAINNET? (yes/no): " CONF; \
	if [ "$$CONF" != "yes" ]; then \
	  echo "Aborting mainnet deployment."; \
	  exit 1; \
	fi; \
	@echo "Uploading contract..."
	soroban contract upload \
		--wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
		--network mainnet \
		--source $(ADMIN_SECRET); \
	@echo "Deploying contract..."
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/micro_lending.wasm \
		--network mainnet \
		--source $(ADMIN_SECRET)

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open

# Development setup
dev-setup:
	@echo "Setting up development environment..."
	rustup target add wasm32-unknown-unknown
	cargo install soroban-cli
	@echo "Development setup completed!"

# Help target
help:
	@echo "Available targets:"
	@echo "  build         - Build the contract"
	@echo "  test          - Run tests"
	@echo "  clean         - Clean build artifacts"
	@echo "  deploy-testnet - Deploy to testnet (requires ADMIN_SECRET)"
	@echo "  deploy-mainnet - Deploy to mainnet (requires ADMIN_SECRET)"
	@echo "  docs          - Generate documentation"
	@echo "  dev-setup     - Setup development environment"
	@echo "  help          - Show this help"
