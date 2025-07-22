# Supply Chain Tracking Contract 🌾

A complete smart contract for tracking the lifecycle of agricultural products from production to final sale on the Stellar network, ensuring transparency and traceability for farmers and consumers.

## 🎯 Overview

This contract enables comprehensive supply chain tracking with:

- **Product Registration**: Register agricultural products with unique identifiers
- **Stage Tracking**: Record key stages (planting, harvesting, processing, transport, sale) with timestamps
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
├── tracking.rs      # Stage management and supply chain tracking
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
    name: String,                  // Stage name (e.g., "Harvesting")
    timestamp: u64,                // When stage occurred
    location: String,              // Geographic location
    data_hash: BytesN<32>,         // Hash of off-chain data
}
```

## 🔑 Core Functions

### Mandatory Functions

- `register_product()` – Register a new agricultural product with initial details
- `add_stage()` – Record a new stage in the product's lifecycle
- `verify_authenticity()` – Validate product authenticity against recorded data
- `get_product_trace()` – Retrieve the full lifecycle of a product
- `link_certificate()` – Associate a product with a certification

### Extended Functions

- `get_product_details()` – Get detailed information about a specific product
- `list_products_by_farmer()` – List all products for a specific farmer
- `generate_qr_code()` – Generate QR code for consumer access
- `trace_by_qr_code()` – Get product trace using QR code
- `validate_stage_transition()` – Validate stage transition logic

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
// Stage 1: Harvesting
contract.add_stage(
    product_id,
    "Harvesting",
    "Field 3, Farm A",
    handler_address,
    harvest_data_hash,
);

// Stage 2: Processing
contract.add_stage(
    product_id,
    "Processing",
    "Processing Plant B",
    processor_address,
    processing_data_hash,
);

// Stage 3: Packaging
contract.add_stage(
    product_id,
    "Packaging",
    "Packaging Facility C",
    packager_address,
    packaging_data_hash,
);
```

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
- ✅ Stage addition and sequencing
- ✅ Supply chain traceability
- ✅ Certificate linking
- ✅ QR code generation and resolution
- ✅ Authentication and error handling
- ✅ Edge cases and invalid inputs

## 🛡 Security

- **Authentication**: All operations require proper authorization
- **Immutability**: Stage records cannot be modified once added
- **Validation**: Input validation prevents invalid data
- **Hash Verification**: Cryptographic verification of data integrity

## 📈 Future Enhancements

- Advanced stage validation rules
- Multi-signature approvals for critical stages
- Integration with IoT sensors for automated stage updates
- Advanced analytics and supply chain metrics
- Support for batch operations and bulk imports

## 📄 License

This project follows the same license as the parent repository.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run the full test suite: `make ci`
5. Submit a pull request

## 📞 Support

For questions or issues:
    - Check the test suite for usage examples
    - Review the code documentation
    - Open an issue in the repository
  