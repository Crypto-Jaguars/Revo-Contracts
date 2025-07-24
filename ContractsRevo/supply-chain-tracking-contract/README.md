# Supply Chain Tracking Contract 🌾

A complete smart contract for tracking the lifecycle of agricultural products from production to final sale on the Stellar network, ensuring transparency and traceability for farmers and consumers.

## 🎯 Overview

This contract enables comprehensive supply chain tracking with:

- **Product Registration**: Register agricultural products with unique identifiers
- **Stage Tier Validation**: Enforce strict sequential progression through agricultural supply chain stages
- **Stage Tracking**: Record key stages with tier validation, timestamps, and cryptographic verification
- **Authenticity Verification**: Validate product authenticity and prevent fraud
- **Certificate Integration**: Link to existing certifications from certificate-management-contract
- **Consumer Access**: Generate QR codes for consumer access to traceability data
- **Data Optimization**: Store critical data on-chain, reference detailed data off-chain via IPFS hashes

## 🏗 Architecture

```txt
supply-chain-tracking-contract/src/
├── lib.rs           # Main contract and exports
├── datatypes.rs     # Data structures and error types
├── product.rs       # Product registration and management
├── tracking.rs      # Stage management and supply chain tracking with stage tier validation
├── validation.rs    # Authenticity verification and certificate linking
├── utils.rs         # Utilities for hash generation and QR codes
└── test.rs          # Comprehensive test suite
```

## 📦 Key Data Structures

### Product

```rust
struct Product {
    product_id: BytesN<32>,        // Unique product identifier
    farmer_id: Address,            // Producer address
    stages: Vec<Stage>,            // All stages embedded in product
    certificate_id: Option<BytesN<32>>, // Linked certification
}
```

### Stage

```rust
struct Stage {
    stage_id: u32,                 // Sequential stage number
    tier: StageTier,               // Validated agricultural tier
    name: String,                  // Stage name (e.g., "Harvesting")
    timestamp: u64,                // When stage occurred
    location: String,              // Geographic location
    data_hash: BytesN<32>,         // Hash of off-chain data
}
```

### Stage Tier

```rust
enum StageTier {
    Planting = 1,       // Seeds/planting stage
    Cultivation = 2,    // Growing/nurturing stage
    Harvesting = 3,     // Harvest/collection stage
    Processing = 4,     // Initial processing/cleaning
    Packaging = 5,      // Packaging for distribution
    Storage = 6,        // Storage/warehousing
    Transportation = 7, // Transport to distribution centers
    Distribution = 8,   // Distribution to retailers
    Retail = 9,         // Retail/market stage
    Consumer = 10,      // Final consumer stage
}
```

### Validation Functions

- `get_current_tier()` – Get the current stage tier for a product
- `get_next_expected_tier()` – Get the next valid tier in the progression
- `validate_tier_progression()` – Internal validation logic

## 🔑 Core Functions

### Mandatory Functions

- `register_product()` – Register a new agricultural product with initial details
- `add_stage(env, product_id, stage_tier, stage_name, location, handler, data_hash)` – Record a new stage with tier validation
- `verify_authenticity()` – Validate product authenticity against recorded data
- `get_product_trace()` – Retrieve the full lifecycle of a product with tier information
- `link_certificate()` – Associate a product with a certification

### Extended Functions

- `get_product_details()` – Get detailed information about a specific product
- `list_products_by_farmer()` – List all products for a specific farmer
- `generate_qr_code()` – Generate QR code for consumer access
- `trace_by_qr_code()` – Get product trace using QR code
- `validate_stage_transition()` – Validate stage transition logic
- `get_current_tier()` – Get current stage tier for a product
- `get_next_expected_tier()` – Get next expected tier in progression

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust and Stellar CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install stellar-cli
rustup target add wasm32-unknown-unknown
```

### Build and Test

```bash
# Clone and navigate to contract
cd supply-chain-tracking-contract

# Run full CI pipeline
make ci

# Or run individual commands
make build          # Build the contract
make test           # Run tests
make stellar-build  # Build with Stellar CLI
```

## 📋 Usage Examples

### 1. Register a Product

```rust
let product_id = contract.register_product(
    farmer_address,
    "Organic Tomatoes",     // product_type
    "BATCH_001",            // batch_number
    "Farm A, California",   // origin_location
    metadata_hash,          // IPFS hash of detailed data
);
```

### 2. Add Supply Chain Stages

```rust
// Stage 1: Must start with Planting tier
contract.add_stage(
    product_id,
    StageTier::Planting,        // Tier validation - must be first
    "Seed Planting",
    "Field 3, Farm A",
    handler_address,
    planting_data_hash,
);

// Stage 2: Must follow with Cultivation tier
contract.add_stage(
    product_id,
    StageTier::Cultivation,     // Next tier in sequence
    "Crop Growing",
    "Field 3, Farm A",
    handler_address,
    cultivation_data_hash,
);

// Stage 3: Harvesting tier
contract.add_stage(
    product_id,
    StageTier::Harvesting,      // Next tier in sequence
    "Crop Harvesting",
    "Field 3, Farm A",
    handler_address,
    harvest_data_hash,
);
```

### 3. Validate Tier Progression

```rust
// Check current tier
let current_tier = contract.get_current_tier(product_id);
// Returns: Some(StageTier::Harvesting)

// Check next expected tier
let next_tier = contract.get_next_expected_tier(product_id);
// Returns: Some(StageTier::Processing)

// Add next stage in correct progression
contract.add_stage(
    product_id,
    StageTier::Processing,      // Matches expected next tier
    "Initial Processing",
    "Processing Plant B",
    processor_address,
    processing_data_hash,
);
```

#### Tier Validation Rules

✅ **Sequential Progression**: Must follow exact tier order (1→2→3→...→10)  
✅ **No Skipping**: Cannot jump from Planting to Processing  
✅ **No Backwards**: Cannot go from Harvesting back to Cultivation  
✅ **Duplicate Prevention**: Cannot add the same tier twice  
✅ **Complete Lifecycle**: Supports full farm-to-consumer tracking  

### 3. Link Certification

```rust
contract.link_certificate(
    product_id,
    organic_certificate_id,
    certification_authority,
);
```

### 4. Generate Consumer QR Code

```rust
let qr_code = contract.generate_qr_code(product_id);
// QR code format: "stellar-supply-chain:{hex_product_id}"
```

### 5. Consumer Traceability

```rust
// Consumer scans QR code and gets full trace
let (product, stages) = contract.trace_by_qr_code(qr_code);

// Access complete supply chain history
for stage in stages {
    println!("Stage {}: {} at {} on {}", 
        stage.stage_id, 
        stage.name, 
        stage.location, 
        stage.timestamp
    );
}
```

## 🔧 Development

### Available Make Commands

```bash
make build         # Build the contract
make test          # Run tests
make stellar-build # Build with Stellar CLI
make lint          # Run linter
make fmt           # Format code
make clean         # Clean artifacts
make ci            # Full CI pipeline
make docs          # Generate documentation
make help          # Show all commands
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_register_product

# Run with output
cargo test -- --nocapture
```

## 🏭 Production Considerations

### On-Chain vs Off-Chain Data

- **On-Chain**: Product IDs, stage hashes, timestamps, locations, certificate links
- **Off-Chain**: Detailed photos, documents, extensive metadata (stored on IPFS)
- **Verification**: All off-chain data referenced by cryptographic hashes

### Scalability Features

- Efficient storage using embedded stages in products
- Indexed access by farmer and product type
- QR code mapping for O(1) consumer lookups
- Optimized for thousands of products and stages

### Security Features

- Authentication required for all state-changing operations
- Immutable stage records once added
- Hash chain verification for supply chain integrity
- Certificate authority validation

## 🔗 Integration

### Certificate Management

The contract integrates with `certificate-management-contract` for:

- Certificate existence validation
- Authority-based certificate linking
- Cross-contract verification calls

### Consumer Applications

QR codes enable easy integration with:

- Mobile apps for consumers
- Web interfaces for traceability
- Third-party verification systems
- Marketplace integrations

## 📊 Events

The contract emits events for:

- `product_registered` - New product registration
- `stage_added` - New stage in supply chain
- `certificate_linked` - Certificate association

## 🧪 Testing

Comprehensive test suite covers:

- ✅ Product registration and validation
- ✅ Stage tier validation and progression rules
- ✅ Wrong tier progression attempts (comprehensive error testing)
- ✅ Sequential tier enforcement
- ✅ Duplicate tier prevention
- ✅ Complete lifecycle validation (all 10 tiers)
- ✅ Edge cases for tier validation
- ✅ Supply chain traceability with tier information
- ✅ Certificate linking
- ✅ QR code generation and resolution
- ✅ Authentication and error handling
- ✅ Backwards progression prevention
- ✅ Edge cases and invalid inputs

## 🛡 Security & Validation

- **Authentication**: All operations require proper authorization
- **Immutability**: Stage records cannot be modified once added
- **Tier Validation**: Strict enforcement of agricultural supply chain progression
- **Input Validation**: Prevents invalid data and tier sequences
- **Hash Verification**: Cryptographic verification of data integrity
- **Sequential Integrity**: Ensures logical supply chain progression

### Error Handling

```rust
enum SupplyChainError {
    UnauthorizedAccess = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    CertificateNotFound = 4,
    ProductNotFound = 5,
    StageNotFound = 6,
    InvalidInput = 7,
    InvalidHash = 8,
    // ... other errors
}
```

## 📈 Future Enhancements

- **IoT Integration**: Automated tier progression with sensor data
- **Multi-signature**: Approvals for critical tier transitions
- **Analytics**: Supply chain metrics and tier timing analysis
- **Batch Operations**: Bulk tier progression for large harvests
- **Quality Gates**: Quality checks required for tier progression
- **Conditional Progression**: Weather or quality-based tier validation

## 📊 Production Benefits

### For Farmers

- **Compliance**: Ensures proper agricultural process documentation
- **Certification**: Links to organic/quality certifications at each tier
- **Traceability**: Complete farm-to-consumer tracking

### For Consumers

- **Transparency**: Full visibility into product journey
- **Trust**: Cryptographically verified supply chain integrity
- **Quality**: Assurance of proper agricultural processes

### For Regulators

- **Audit Trail**: Immutable record of agricultural processes
- **Compliance**: Ensures proper food safety protocols
- **Verification**: Cryptographic proof of supply chain integrity
