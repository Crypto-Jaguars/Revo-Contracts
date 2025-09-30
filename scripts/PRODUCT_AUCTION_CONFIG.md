# Product Auction Contract Deployment Configuration

This file provides examples of how to configure your environment for deploying the product auction contract.

## Prerequisites Setup

### 1. Install Stellar CLI

```bash
# Install Stellar CLI
cargo install stellar-cli

# Verify installation
stellar --version
```

### 2. Install jq (JSON processor)

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq

# CentOS/RHEL
sudo yum install jq

# Verify installation
jq --version
```

### 3. Configure Stellar Profile

```bash
# Add a new profile for testnet
stellar config keys add testnet-profile

# Add a new profile for mainnet
stellar config keys add mainnet-profile

# List configured profiles
stellar config keys list

# Set default profile (optional)
stellar config --profile testnet-profile
```

## Example Deployment Commands

### Testnet Deployment

```bash
# Basic testnet deployment with default profile
cd /Users/villarley/Documents/GitHub/Revo-Contracts/scripts
./deploy_product_auction.zsh testnet

# Testnet deployment with specific profile
./deploy_product_auction.zsh testnet my-testnet-profile
```

### Mainnet Deployment

```bash
# Mainnet deployment (be careful!)
./deploy_product_auction.zsh mainnet production-profile
```

## Expected Output Structure

After successful deployment, you should see:

```
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
```

## Files Created

The deployment script creates the following files in `ContractsRevo/product-auction-contract/logs/`:

1. **deployment_YYYYMMDD_HHMMSS.log** - Detailed deployment log
2. **deployment_results.json** - JSON file with deployment metadata
3. **latest_deployment.txt** - Human-readable summary

## Example deployment_results.json

```json
{
  "contract_name": "product-auction-contract",
  "network": "testnet",
  "profile": "default",
  "wasm_hash": "DD4C3E9F8B2A1C5D6E7F8A9B0C1D2E3F4A5B6C7D8E9F0A1B2C3D4E5F6A7B8C9D0",
  "contract_id": "CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN",
  "deployment_timestamp": "2024-12-01 14:30:22 UTC",
  "wasm_path": "/Users/villarley/Documents/GitHub/Revo-Contracts/ContractsRevo/product-auction-contract/target/wasm32-unknown-unknown/release/product_auction_contract.wasm",
  "deployment_log": "/Users/villarley/Documents/GitHub/Revo-Contracts/ContractsRevo/product-auction-contract/logs/deployment_20241201_143022.log"
}
```

## Contract Functionality Overview

The Product Auction Contract provides:

### 🏷️ Product Listings
- Create auction listings for agricultural products
- Set starting and reserve prices
- Define auction duration
- Add product metadata and descriptions

### 🎯 Bidding System
- Place bids on active auctions
- Automatic bid validation
- Bid increment enforcement
- Real-time auction updates

### ⏰ Auction Management
- Automated auction lifecycle management
- Start and end auction events
- Settlement processing
- Winner determination

### 🚚 Shipping Integration
- Coordinate product delivery
- Shipping cost calculation
- Delivery tracking
- Logistics management

### ✅ Verification System
- Product authenticity verification
- Quality assurance checks
- Certification validation
- Trust and safety measures

## Testing the Deployed Contract

### Initialize Contract

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- initialize
```

### Create Product Listing

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- create_listing \
  --seller <SELLER_ADDRESS> \
  --product_id "PROD001" \
  --product_name "Organic Tomatoes" \
  --starting_price 1000 \
  --reserve_price 500 \
  --duration 86400 \
  --description "Fresh organic tomatoes from local farm"
```

### Place Bid

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- place_bid \
  --listing_id <LISTING_ID> \
  --bidder <BIDDER_ADDRESS> \
  --amount 1200
```

### End Auction

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- end_auction \
  --listing_id <LISTING_ID>
```

### Settle Auction

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <SOURCE_ACCOUNT> \
  --network testnet \
  -- settle_auction \
  --listing_id <LISTING_ID>
```

## Troubleshooting Common Issues

### Issue: "Stellar CLI is not installed"
```bash
# Solution: Install Stellar CLI
cargo install stellar-cli
```

### Issue: "Profile does not exist"
```bash
# Solution: Create a profile
stellar config keys add my-profile
```

### Issue: "jq is not installed"
```bash
# Solution: Install jq
brew install jq  # macOS
sudo apt-get install jq  # Ubuntu
```

### Issue: "Contract build failed"
```bash
# Solution: Check Rust installation and contract dependencies
cd ContractsRevo/product-auction-contract
cargo check
cargo build
```

### Issue: "Insufficient funds"
```bash
# Solution: Fund your account
# For testnet, use the Stellar Testnet Friendbot
curl "https://friendbot.stellar.org/?addr=<YOUR_ACCOUNT_ADDRESS>"
```

## Network-Specific Configuration

### Testnet Configuration
- Network: `testnet`
- Horizon URL: `https://horizon-testnet.stellar.org`
- Friendbot: `https://friendbot.stellar.org`
- Explorer: `https://testnet.stellar.org/explorer`

### Mainnet Configuration
- Network: `mainnet`
- Horizon URL: `https://horizon.stellar.org`
- Explorer: `https://stellar.org/explorer`
- ⚠️ **Warning**: Mainnet deployments cost real XLM

## Security Best Practices

1. **Always test on testnet first**
2. **Use separate profiles for testnet and mainnet**
3. **Keep private keys secure**
4. **Verify contract addresses on Stellar Explorer**
5. **Review deployment logs for any issues**
6. **Test all contract functions before mainnet deployment**

## Integration with CI/CD

The deployment script is designed for CI/CD integration:

```yaml
# Example GitHub Actions workflow
name: Deploy Product Auction Contract
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install Stellar CLI
        run: cargo install stellar-cli
      - name: Install jq
        run: sudo apt-get install jq
      - name: Deploy to Testnet
        run: ./scripts/deploy_product_auction.zsh testnet ci-profile
        env:
          STELLAR_PROFILE: ${{ secrets.STELLAR_PROFILE }}
```

## Contract Architecture

The Product Auction Contract is built with modular architecture:

```
src/
├── lib.rs              # Main contract entry point
├── datatype.rs         # Data structures and types
├── interfaces.rs       # External interfaces
├── listing.rs          # Product listing management
├── product_auction.rs  # Core auction logic
├── shipping.rs         # Shipping and logistics
├── verification.rs     # Product verification
└── test.rs            # Contract tests
```

## Use Cases and Applications

### 🚜 Agricultural Marketplace
- Direct farmer-to-consumer sales
- Seasonal produce auctions
- Bulk commodity trading
- Organic product verification

### 🏪 Retail Integration
- Store-to-store auctions
- Inventory management
- Supply chain optimization
- Quality assurance tracking

### 🌱 Sustainable Agriculture
- Carbon credit auctions
- Sustainable farming incentives
- Environmental impact tracking
- Certification verification

## Support and Resources

- **Documentation**: See PRODUCT_AUCTION_README.md
- **Testing**: Use test_product_auction_deploy.zsh
- **Stellar Docs**: https://developers.stellar.org/docs
- **Contract Explorer**: Use Stellar Explorer for network-specific URLs
