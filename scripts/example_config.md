# Example Configuration for Water Management Contract Deployment

This file provides examples of how to configure your environment for deploying the water management contract.

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
./deploy_water_management.zsh testnet

# Testnet deployment with specific profile
./deploy_water_management.zsh testnet my-testnet-profile
```

### Mainnet Deployment

```bash
# Mainnet deployment (be careful!)
./deploy_water_management.zsh mainnet production-profile
```

## Expected Output Structure

After successful deployment, you should see:

```
================================
DEPLOYMENT COMPLETED
================================
✅ Contract deployed successfully!

Deployment Details:
  Contract: water-management-contract
  Network: testnet
  Profile: default
  WASM Hash: CC3B2F8A9D7E6F4A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8A9B0C1D2E3F4
  Contract ID: CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN
```

## Files Created

The deployment script creates the following files in `ContractsRevo/water-management-contract/logs/`:

1. **deployment_YYYYMMDD_HHMMSS.log** - Detailed deployment log
2. **deployment_results.json** - JSON file with deployment metadata
3. **latest_deployment.txt** - Human-readable summary

## Example deployment_results.json

```json
{
  "contract_name": "water-management-contract",
  "network": "testnet",
  "profile": "default",
  "wasm_hash": "CC3B2F8A9D7E6F4A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8A9B0C1D2E3F4",
  "contract_id": "CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN",
  "deployment_timestamp": "2024-12-01 14:30:22 UTC",
  "wasm_path": "/Users/villarley/Documents/GitHub/Revo-Contracts/ContractsRevo/water-management-contract/target/wasm32-unknown-unknown/release/water_management_contract.wasm",
  "deployment_log": "/Users/villarley/Documents/GitHub/Revo-Contracts/ContractsRevo/water-management-contract/logs/deployment_20241201_143022.log"
}
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
cd ContractsRevo/water-management-contract
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

### Mainnet Configuration
- Network: `mainnet`
- Horizon URL: `https://horizon.stellar.org`
- ⚠️ **Warning**: Mainnet deployments cost real XLM

## Security Best Practices

1. **Always test on testnet first**
2. **Use separate profiles for testnet and mainnet**
3. **Keep private keys secure**
4. **Verify contract addresses on Stellar Explorer**
5. **Review deployment logs for any issues**

## Integration with CI/CD

The deployment script is designed for CI/CD integration:

```yaml
# Example GitHub Actions workflow
name: Deploy Water Management Contract
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
        run: ./scripts/deploy_water_management.zsh testnet ci-profile
        env:
          STELLAR_PROFILE: ${{ secrets.STELLAR_PROFILE }}
```
