# Supply Chain Tracking Contract Makefile
# Build and deployment automation

# Default target
default: build

# Build all targets
all: build stellar-build

# Build the contract
build:
	@echo "Building supply chain tracking contract..."
	cargo build --target wasm32-unknown-unknown --release
	@echo "Build completed successfully!"

# Build with Stellar CLI
stellar-build:
	@echo "Building contract with Stellar CLI..."
	stellar contract build
	@echo "Stellar build completed successfully!"

# Run tests
test:
	@echo "Running tests..."
	cargo test
	@echo "All tests passed!"

# Run tests with output
test-verbose:
	@echo "Running tests with verbose output..."
	cargo test -- --nocapture
	@echo "Verbose tests completed!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Clean completed!"

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt
	@echo "Code formatting completed!"

# Run linter
lint:
	@echo "Running linter..."
	cargo clippy -- -D warnings
	@echo "Linting completed!"

# Check code without building
check:
	@echo "Checking code..."
	cargo check
	@echo "Code check completed!"

# Full CI pipeline
ci: fmt lint check test build stellar-build
	@echo "CI pipeline completed successfully!"

# Deploy contract (placeholder for deployment scripts)
deploy:
	@echo "Deploying contract..."
	@echo "Note: Add your deployment commands here"
	@echo "Example: stellar contract deploy --wasm target/wasm32-unknown-unknown/release/supply_chain_tracking_contract.wasm"

# Deploy to Stellar testnet
deploy-testnet: build
	@echo "Deploying Supply Chain Tracking Contract to Stellar testnet..."
	@echo "Installing contract WASM..."
	stellar contract install \
		--wasm target/wasm32-unknown-unknown/release/supply_chain_tracking_contract.wasm \
		--network testnet
	@echo "Deploying contract..."
	stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/supply_chain_tracking_contract.wasm \
		--network testnet

# Deploy to Stellar testnet with custom source
ifeq ($(strip $(STELLAR_SOURCE)),)
deploy-testnet-custom:
	$(error STELLAR_SOURCE environment variable is not set. Usage: make deploy-testnet-custom STELLAR_SOURCE=your_secret_key)
else
deploy-testnet-custom: build
	@echo "Deploying Supply Chain Tracking Contract to Stellar testnet with custom source..."
	@echo "Installing contract WASM..."
	stellar contract install \
		--wasm target/wasm32-unknown-unknown/release/supply_chain_tracking_contract.wasm \
		--network testnet \
		--source $(STELLAR_SOURCE)
	@echo "Deploying contract..."
	stellar contract deploy \
		--wasm target/wasm32-unknown-unknown/release/supply_chain_tracking_contract.wasm \
		--network testnet \
		--source $(STELLAR_SOURCE)
endif

# Generate docs
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open
	@echo "Documentation generated!"

# Watch for changes and rebuild
watch:
	@echo "Watching for changes..."
	cargo watch -x build

# Development setup
dev-setup:
	@echo "Setting up development environment..."
	rustup target add wasm32-unknown-unknown
	cargo install stellar-cli
	@echo "Development setup completed!"

# Help target
help:
	@echo "Available targets:"
	@echo "  build         - Build the contract"
	@echo "  stellar-build - Build with Stellar CLI"
	@echo "  test          - Run tests"
	@echo "  test-verbose  - Run tests with verbose output"
	@echo "  clean         - Clean build artifacts"
	@echo "  fmt           - Format code"
	@echo "  lint          - Run linter"
	@echo "  check         - Check code without building"
	@echo "  ci            - Run full CI pipeline"
	@echo "  deploy        - Deploy contract"
	@echo "  docs          - Generate documentation"
	@echo "  watch         - Watch for changes and rebuild"
	@echo "  dev-setup     - Set up development environment"
	@echo "  help          - Show this help message"

.PHONY: default build stellar-build test test-verbose clean fmt lint check ci deploy docs watch dev-setup help