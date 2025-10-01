# Makefile for Farmer Staking Contract

.PHONY: all build test clean deploy help

# Default target
all: build test

# Build the contract
build:
	@echo "Building farmer staking contract..."
	@stellar contract build
	@echo "Build complete! WASM file generated in target/wasm32-unknown-unknown/release/"

# Run tests
test:
	@echo "Running tests..."
	@cargo test
	@echo "All tests passed!"

# Run tests with output
test-verbose:
	@echo "Running tests with output..."
	@cargo test -- --nocapture

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/
	@echo "Clean complete!"

# Deploy to testnet (requires environment variables)
deploy-testnet:
	@echo "Deploying to testnet..."
	@stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/farmer_staking_contract.wasm \
		--source $(SOURCE_ACCOUNT) \
		--network testnet

# Deploy to mainnet (requires environment variables)
deploy-mainnet:
	@echo "WARNING: Deploying to mainnet!"
	@read -p "Are you sure? (yes/no): " confirm && [ "$$confirm" = "yes" ] || exit 1
	@stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/farmer_staking_contract.wasm \
		--source $(SOURCE_ACCOUNT) \
		--network mainnet

# Install Stellar CLI and dependencies
install-deps:
	@echo "Installing dependencies..."
	@rustup target add wasm32-unknown-unknown
	@cargo install --locked stellar-cli --features opt
	@echo "Dependencies installed!"

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt
	@echo "Code formatted!"

# Run clippy lints
lint:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings
	@echo "Linting complete!"

# Check contract size
size:
	@echo "Checking contract size..."
	@ls -lh target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null || echo "Contract not built yet. Run 'make build' first."

# Generate documentation
docs:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open

# Run a specific test
test-single:
	@echo "Running test: $(TEST_NAME)"
	@cargo test $(TEST_NAME) -- --nocapture

# Optimize WASM for deployment
optimize:
	@echo "Optimizing WASM..."
	@stellar contract optimize --wasm target/wasm32-unknown-unknown/release/farmer_staking_contract.wasm
	@echo "Optimization complete!"

# Build and test together
check: build test lint
	@echo "All checks passed!"

# Help target
help:
	@echo "Farmer Staking Contract - Available targets:"
	@echo ""
	@echo "  make build         - Build the contract"
	@echo "  make test          - Run all tests"
	@echo "  make test-verbose  - Run tests with output"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make deploy-testnet - Deploy to testnet (requires SOURCE_ACCOUNT env var)"
	@echo "  make deploy-mainnet - Deploy to mainnet (requires SOURCE_ACCOUNT env var)"
	@echo "  make install-deps  - Install Stellar CLI and dependencies"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run clippy lints"
	@echo "  make size          - Check contract size"
	@echo "  make docs          - Generate documentation"
	@echo "  make optimize      - Optimize WASM for deployment"
	@echo "  make check         - Build, test, and lint"
	@echo "  make test-single TEST_NAME=<test_name> - Run a specific test"
	@echo ""
	@echo "Example deployment:"
	@echo "  SOURCE_ACCOUNT=your_account_here make deploy-testnet"
