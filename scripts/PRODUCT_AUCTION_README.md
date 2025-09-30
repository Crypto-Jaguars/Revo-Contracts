# Product Auction Contract Deployment

This directory contains the automated deployment script for the Product Auction Contract to Stellar networks.

## 🏗️ Product Auction Contract Deployment Script

The `deploy_product_auction.zsh` script automates the complete deployment process for the product auction contract to Stellar networks (testnet/mainnet).

### ✨ Features

- ✅ Automated build, upload, and deploy process
- ✅ Network selection (testnet/mainnet)
- ✅ Profile-based deployment
- ✅ Comprehensive error handling and validation
- ✅ Detailed logging with timestamps
- ✅ JSON and text result files
- ✅ Colored output for better readability
- ✅ Prerequisites validation

### 🎯 Contract Overview

The Product Auction Contract enables:
- **Product Listings**: Create and manage auction listings for agricultural products
- **Bidding System**: Allow users to place bids on auctioned items
- **Auction Management**: Handle auction lifecycle (start, end, settlement)
- **Shipping Integration**: Coordinate product delivery and logistics
- **Verification System**: Ensure product quality and authenticity

### 📋 Prerequisites

Before running the deployment script, ensure you have:

1. **Stellar CLI** installed:
   ```bash
   cargo install stellar-cli
   ```

2. **jq** installed (for JSON parsing):
   ```bash
   # macOS
   brew install jq
   
   # Ubuntu/Debian
   apt-get install jq
   
   # CentOS/RHEL
   yum install jq
   ```

3. **Rust and Cargo** installed:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

4. **Stellar Profile** configured:
   ```bash
   stellar config keys add <profile-name>
   ```

### 🚀 Usage

```bash
./deploy_product_auction.zsh [network] [profile]
```

#### Parameters

- `network` (required): Target network - `testnet` or `mainnet`
- `profile` (optional): Stellar profile to use (defaults to `default`)

#### Examples

```bash
# Deploy to testnet with default profile
./deploy_product_auction.zsh testnet

# Deploy to testnet with custom profile
./deploy_product_auction.zsh testnet my_testnet_profile

# Deploy to mainnet with production profile
./deploy_product_auction.zsh mainnet production

# Show help
./deploy_product_auction.zsh --help
```

### 🔧 What the Script Does

1. **Prerequisites Check**: Validates that all required tools are installed
2. **Parameter Validation**: Ensures network and profile parameters are valid
3. **Contract Build**: Uses `stellar contract build` to compile the contract
4. **Contract Upload**: Uploads the WASM file and captures the WASM hash
5. **Contract Deploy**: Deploys the contract and captures the contract ID
6. **Result Saving**: Saves deployment results in multiple formats
7. **Logging**: Creates detailed logs with timestamps

### 📊 Output Files

The script creates several files in the contract's `logs/` directory:

- `deployment_YYYYMMDD_HHMMSS.log` - Detailed deployment log
- `deployment_results.json` - JSON file with deployment metadata
- `latest_deployment.txt` - Human-readable summary

### 📸 Example Output

```
================================
BUILDING CONTRACT
================================
[INFO] Building contract with profile: default
[SUCCESS] Contract built successfully
[INFO] WASM file: /path/to/product_auction_contract.wasm
[INFO] WASM size: 2.3M

================================
UPLOADING CONTRACT
================================
[INFO] Uploading contract to testnet network
[SUCCESS] Contract uploaded successfully
[INFO] WASM Hash: DD4C3E9F8B2A1C5D6E7F8A9B0C1D2E3F4A5B6C7D8E9F0A1B2C3D4E5F6A7B8C9D0

================================
DEPLOYING CONTRACT
================================
[INFO] Deploying contract to testnet network
[SUCCESS] Contract deployed successfully
[INFO] Contract ID: CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN

================================
DEPLOYMENT COMPLETED
================================
✅ Contract deployed successfully!

Deployment Details:
  Contract: product-auction-contract
  Network: testnet
  Profile: default
  WASM Hash: DD4C3E9F8B2A1C5D6E7F8A9B0C1D2E3F4A5B6C7D8E9F0A1B2C3D4E5F6A7B8C9D0
  Contract ID: CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN

Files Created:
  Deployment Log: /path/to/logs/deployment_20241201_143022.log
  Results JSON: /path/to/logs/deployment_results.json
  Summary: /path/to/logs/latest_deployment.txt

Next Steps:
  1. Verify deployment on Stellar Explorer
  2. Test contract functionality
  3. Initialize contract if required

Testnet Explorer: https://testnet.stellar.org/explorer
```

### 🛠️ Error Handling

The script includes comprehensive error handling:

- **Prerequisites validation**: Checks for required tools
- **Parameter validation**: Validates network and profile parameters
- **Build validation**: Ensures contract builds successfully
- **Upload validation**: Verifies WASM upload and hash extraction
- **Deploy validation**: Confirms successful deployment and contract ID extraction
- **Exit codes**: Proper exit codes for automation and CI/CD integration

### 🔍 Troubleshooting

#### Common Issues

1. **"Stellar CLI is not installed"**
   ```bash
   cargo install stellar-cli
   ```

2. **"jq is not installed"**
   ```bash
   brew install jq  # macOS
   apt-get install jq  # Ubuntu
   ```

3. **"Profile does not exist"**
   ```bash
   stellar config keys add <profile-name>
   ```

4. **"Contract build failed"**
   - Check Rust installation
   - Verify contract code compiles
   - Check dependencies in Cargo.toml

5. **"WASM file not found"**
   - Ensure build completed successfully
   - Check if WASM file exists in target directory
   - Verify contract name matches expected filename

#### Debug Mode

For debugging, you can run the script with verbose output:

```bash
set -x  # Enable debug mode
./deploy_product_auction.zsh testnet
```

### 🔒 Security Considerations

- **Testnet First**: Always test deployments on testnet before mainnet
- **Profile Security**: Use secure profiles with appropriate permissions
- **Log Files**: Deployment logs may contain sensitive information
- **Network Validation**: Script validates network parameter to prevent accidents

### 🧪 Testing the Deployed Contract

After successful deployment, you can test the contract functionality:

```bash
# Initialize the contract (if required)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- initialize

# Create a product listing
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- create_listing \
  --seller <SELLER_ADDRESS> \
  --product_id "PROD001" \
  --starting_price 1000 \
  --reserve_price 500 \
  --duration 86400

# Place a bid
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- place_bid \
  --listing_id <LISTING_ID> \
  --bidder <BIDDER_ADDRESS> \
  --amount 1200
```

### 🔗 Integration

The script is designed for easy integration with:

- **CI/CD Pipelines**: Proper exit codes and structured output
- **Automation Tools**: JSON output for programmatic processing
- **Monitoring**: Detailed logging for audit trails
- **Documentation**: Automatic result file generation

### 📚 Contract Architecture

The Product Auction Contract consists of several modules:

- **Product Management**: Handle product listings and metadata
- **Auction Engine**: Manage auction lifecycle and bidding
- **Shipping Integration**: Coordinate delivery logistics
- **Verification System**: Ensure product authenticity
- **Interface Layer**: External API for contract interactions

### 🎯 Use Cases

This contract enables various agricultural marketplace scenarios:

1. **Direct Sales**: Farmers can list products for immediate sale
2. **Auction Sales**: Competitive bidding for high-value products
3. **Seasonal Markets**: Time-limited auctions for seasonal produce
4. **Bulk Sales**: Large quantity auctions for wholesale buyers
5. **Quality Assurance**: Verified product listings with quality guarantees

### 📞 Support

For issues or questions:

1. Check the deployment logs for detailed error information
2. Verify all prerequisites are installed and configured
3. Ensure network connectivity and account funding
4. Review Stellar CLI documentation for network-specific requirements
5. Check the contract's test suite for functionality validation

### 🔄 Version History

- **v1.0.0**: Initial deployment script with full automation
- Features: Build, upload, deploy, logging, error handling
- Support: Testnet and mainnet deployment
