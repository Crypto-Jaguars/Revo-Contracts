# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Revo Contracts is a comprehensive collection of Stellar smart contracts focused on agricultural and cooperative ecosystems. The repository implements 24+ interconnected contracts covering auctions, quality management, certifications, tokenization, lending, and cooperative governance using Soroban (Stellar's smart contract platform).

## Common Development Commands

### Building Contracts
```bash
# Build all contracts in the workspace
stellar contract build

# Build specific contract (navigate to contract directory first)
cd ContractsRevo/agricultural-auction-contract
stellar contract build
```

### Testing
```bash
# Run all tests in workspace
cargo test

# Run tests for specific contract
cd ContractsRevo/certificate-management-contract
cargo test

# Run specific test
cargo test test_issue_certification

# Run tests with output
cargo test -- --nocapture
```

### Deployment (Testnet)
```bash
# Deploy contract to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/CONTRACT_NAME.wasm \
  --source SOURCE_ACCOUNT \
  --network testnet

# Interact with deployed contract
stellar contract invoke \
  --id CONTRACT_ID \
  --source SOURCE_ACCOUNT \
  --network testnet \
  -- \
  FUNCTION_NAME \
  --arg1 value1 \
  --arg2 value2
```

### Development Setup
```bash
# Install Rust target for WebAssembly
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

## Architecture Overview

### Contract Organization
The codebase follows a modular architecture with 24+ specialized contracts in the `ContractsRevo/` directory:

**Core Agricultural Contracts:**
- `agricultural-auction-contract`: Farmer-to-buyer auction system with dynamic pricing
- `agricultural-quality-contract`: Quality assurance and dispute resolution
- `agricultural-training-contract`: Training program management and certification
- `certificate-management-contract`: Digital certification issuance and verification
- `commodity-token-contract`: Commodity-backed tokenization

**Financial & Cooperative:**
- `microlending-contract`: Farmer financing and loan management
- `farmer-insurance-contract`: Agricultural insurance products
- `cooperative-management-contract`: Cooperative governance and management
- `cross-cooperative-trade-contract`: Inter-cooperative commerce

**Market & Trading:**
- `product-auction-contract`: Product-specific auction mechanisms
- `price-stabilization-contract`: Price stability mechanisms
- `market-demand-forecasting-contract`: Demand prediction and analytics

### Architectural Patterns

**Modular Contract Structure:**
Each contract follows a consistent pattern:
- `lib.rs` - Main contract interface and public methods
- `datatypes.rs` - Contract-specific types and enums
- `error.rs` - Comprehensive error handling
- Functional modules (`storage.rs`, `validation.rs`, etc.)
- `test.rs` - Comprehensive unit tests

**Error Handling:**
- Uses `#[contracterror]` for contract-specific errors
- Comprehensive error types for each domain (AdminError, AuctionError, ProductError, etc.)
- Result-based error propagation throughout contracts

**Data Management:**
- Storage keys use enum-based organization (`DataKey`)
- Persistent and instance storage patterns
- Type-safe serialization with `#[contracttype]`

**Authentication & Authorization:**
- Address-based authentication using `require_auth()`
- Admin pattern for contract initialization and privileged operations
- Multi-party authorization for sensitive operations

### Cross-Contract Integration

**Shared Types:**
Many contracts share common agricultural domain types:
- `AgriculturalProduct` with quality grades, freshness ratings, and certifications
- `QualityGrade` enum (Premium, GradeA, GradeB, etc.)
- `FreshnessRating` time-based quality assessment
- `StorageCondition` for product handling requirements

**Contract Interconnections:**
- Certificate management integrates with quality verification
- Auction contracts reference product listings and quality assessments
- Token contracts can be backed by certified commodities
- Insurance contracts reference quality and certification data

## Development Guidelines

### Testing Patterns
- Use `setup_test()` helper functions for consistent test environments
- Mock authentication with `env.mock_all_auths()` for convenience
- Test both success and error cases comprehensively
- Use `try_*` method variants for testing expected failures

### Contract Deployment Flow
1. Build contract: `stellar contract build`
2. Deploy to testnet with source account
3. Initialize contract with admin address
4. Test core functions via CLI invocation
5. Verify events and storage state

### Error Handling Best Practices
- Always use Result types for fallible operations
- Provide specific error variants for different failure modes
- Include context in error variants where helpful
- Test error conditions explicitly

### Storage Patterns
- Use instance storage for contract configuration
- Use persistent storage for long-term data
- Implement clear storage key organization via DataKey enums
- Consider storage costs for large data structures

## Contract Interaction Patterns

### Initialization Sequence
Most contracts follow this pattern:
1. `initialize(env, admin)` - Sets contract admin
2. Admin configures contract parameters
3. Users interact with core contract functions
4. Events are emitted for key state changes

### Multi-Contract Workflows
Example: Product Sale with Certification
1. `certificate-management-contract`: Issue quality certification
2. `agricultural-auction-contract`: List certified product for auction
3. `agricultural-quality-contract`: Handle any quality disputes
4. Cross-reference certification data across contracts

## Stellar-Specific Considerations

- All contracts target `#![no_std]` for Soroban compatibility
- Use Soroban SDK types (`Address`, `Symbol`, `Vec`, etc.)
- WebAssembly compilation to `.wasm` files in `target/deploy/`
- Network-specific deployment (testnet vs mainnet)
- Transaction fees and resource usage optimization