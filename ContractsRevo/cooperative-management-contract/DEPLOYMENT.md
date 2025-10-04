# Cooperative Management Contract Deployment Guide

## 📋 Overview
This guide provides detailed instructions for deploying the Cooperative Management Contract to Stellar Testnet or Mainnet using the automated deployment script.

## 🛠️ Prerequisites

Before running the deployment script, ensure you have the following tools installed:

### Required Tools
1. **Stellar CLI**
   ```bash
   cargo install stellar-cli
   ```

2. **Rust and Cargo**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Stellar Identity Setup

You need a configured Stellar identity to deploy the contract:

```bash
# Generate a new identity for testnet
stellar keys generate <identity-name> --network testnet

# Or for mainnet
stellar keys generate <identity-name> --network mainnet

# List available identities
stellar keys ls

# Get identity address
stellar keys address <identity-name>
```

**Important:** For testnet deployments, fund your account using the [Stellar Friendbot](https://friendbot.stellar.org/).

## 🚀 Deployment Script

### Location
The deployment script is located at:
```
/scripts/deploy_cooperative_management.zsh
```

### Usage

#### Basic Syntax
```bash
./scripts/deploy_cooperative_management.zsh [network] [identity]
```

#### Parameters
- **network** (required): Target network - either `testnet` or `mainnet`
- **identity** (optional): Stellar identity/key to use (defaults to `default` if not specified)

### Examples

#### Deploy to Testnet with Default Identity
```bash
./scripts/deploy_cooperative_management.zsh testnet
```

#### Deploy to Testnet with Specific Identity
```bash
./scripts/deploy_cooperative_management.zsh testnet alice
```

#### Deploy to Mainnet with Production Identity
```bash
./scripts/deploy_cooperative_management.zsh mainnet production
```

#### Show Help Information
```bash
./scripts/deploy_cooperative_management.zsh --help
```

## 📊 Deployment Process

The script performs the following steps automatically:

### 1. Prerequisites Check
- Verifies Stellar CLI is installed
- Verifies Cargo is installed
- Validates network parameter
- Validates identity exists

### 2. Contract Build
```bash
stellar contract build --profile release
```
- Builds the contract using release profile
- Generates optimized WASM binary
- Output: `target/wasm32v1-none/release/cooperative_management_contract.wasm`

### 3. Contract Upload
```bash
stellar contract upload --source-account <identity> --network <network> --wasm <path>
```
- Uploads WASM to Stellar network
- Parses `wasm_hash` from output
- Output: 64-character hexadecimal hash

### 4. Contract Deployment
```bash
stellar contract deploy --source-account <identity> --network <network> --wasm-hash <hash>
```
- Deploys contract instance from WASM hash
- Parses `contract_id` from output
- Output: Stellar contract ID (starts with 'C', 56 characters)

### 5. Results Saving
The script saves deployment results to:
- **JSON Results:** `ContractsRevo/cooperative-management-contract/logs/deployment_results.json`
- **Text Summary:** `ContractsRevo/cooperative-management-contract/logs/latest_deployment.txt`
- **Detailed Log:** `ContractsRevo/cooperative-management-contract/logs/deployment_YYYYMMDD_HHMMSS.log`

## 📁 Output Files

### Deployment Results JSON
Location: `logs/deployment_results.json`

```json
{
  "contract_name": "cooperative-management-contract",
  "network": "testnet",
  "profile": "default",
  "wasm_hash": "a1b2c3d4...",
  "contract_id": "CABC123...",
  "deployment_timestamp": "2025-10-02 12:34:56 UTC",
  "wasm_path": "/path/to/wasm",
  "deployment_log": "/path/to/log"
}
```

### Deployment Summary
Location: `logs/latest_deployment.txt`

```
Cooperative Management Contract Deployment Summary
================================================
Contract: cooperative-management-contract
Network: testnet
Profile: default
WASM Hash: a1b2c3d4e5f6...
Contract ID: CABC123XYZ...
Deployed: 2025-10-02 12:34:56 UTC
Log File: /path/to/deployment.log
```

### Deployment Log
Location: `logs/deployment_YYYYMMDD_HHMMSS.log`

Contains timestamped details of each deployment step, including:
- Build output
- Upload response
- Deployment response
- Any errors or warnings

## ✅ Post-Deployment Steps

After successful deployment, follow these steps:

### 1. Verify Deployment
Check the contract on Stellar Explorer:
- **Testnet:** https://testnet.stellar.org/explorer
- **Mainnet:** https://stellar.org/explorer

Search for your contract ID to verify it's deployed.

### 2. Initialize the Contract
The Cooperative Management Contract requires initialization before use:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <identity> \
  --network <network> \
  -- \
  initialize \
  --admin <ADMIN_ADDRESS> \
  --cooperative_name <NAME> \
  --description <DESCRIPTION>
```

Replace:
- `<CONTRACT_ID>`: Your deployed contract ID
- `<identity>`: Your Stellar identity name
- `<network>`: testnet or mainnet
- `<ADMIN_ADDRESS>`: Address of the contract administrator
- `<NAME>`: Cooperative name (as Symbol)
- `<DESCRIPTION>`: Cooperative description (as String)

### 3. Test Contract Functionality
Test basic contract functions to ensure proper deployment:

```bash
# Get cooperative info
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <identity> \
  --network <network> \
  -- \
  get_cooperative_info
```

## 🔧 Troubleshooting

### Common Issues

#### 1. "Stellar CLI is not installed"
**Solution:** Install Stellar CLI
```bash
cargo install stellar-cli
```

#### 2. "Identity does not exist"
**Solution:** Create a new identity
```bash
stellar keys generate <name> --network <network>
```

#### 3. "WASM file not found"
**Solution:** Ensure the contract builds successfully. Check:
- You're running the script from the correct directory
- The contract builds without errors
- Rust and Cargo are properly installed

#### 4. "Contract upload failed"
**Possible causes:**
- Insufficient balance (testnet: use Friendbot)
- Network issues
- Invalid identity

**Solution:**
```bash
# Fund testnet account
curl "https://friendbot.stellar.org/?addr=<YOUR_ADDRESS>"

# Check identity
stellar keys ls
stellar keys address <identity-name>
```

#### 5. "Failed to extract WASM hash/contract ID"
**Solution:** Check the deployment log for details:
```bash
cat ContractsRevo/cooperative-management-contract/logs/deployment_*.log
```

### Error Codes
The script uses exit codes to indicate failure:
- `0`: Success
- `1`: General error (check logs for details)

## 🔒 Security Best Practices

### For Testnet Deployments
- Use separate identities for testing
- Don't reuse testnet keys for mainnet
- Regularly rotate test identities

### For Mainnet Deployments
- **CRITICAL:** Ensure sufficient XLM balance for deployment costs
- Use hardware wallet or secure key storage
- Test thoroughly on testnet first
- Keep private keys secure and backed up
- Use dedicated production identities
- Review contract code before deployment
- Document contract addresses securely
- Set up monitoring for deployed contracts

### Key Management
```bash
# Export identity (keep secure!)
stellar keys show <identity-name>

# Never commit private keys to version control
# Add to .gitignore:
# *.secret
# *.key
# .stellar/
```

## 📈 Monitoring Deployment

### Check Deployment Status
```bash
# View latest deployment
cat ContractsRevo/cooperative-management-contract/logs/latest_deployment.txt

# View all deployments
ls -lh ContractsRevo/cooperative-management-contract/logs/

# View latest log
tail -f ContractsRevo/cooperative-management-contract/logs/deployment_*.log
```

### Verify Contract on Network
```bash
# Get contract info
stellar contract inspect --id <CONTRACT_ID> --network <network>

# Check contract functions
stellar contract inspect --id <CONTRACT_ID> --network <network>
```

## 📚 Additional Resources

- [Stellar Developer Documentation](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)
- [Contract README](./README.md)

## 🆘 Getting Help

If you encounter issues not covered in this guide:

1. Check the deployment logs in `logs/deployment_*.log`
2. Review the [Contract README](./README.md)
3. Consult [Stellar Discord](https://discord.gg/stellar)
4. Review [Soroban Examples](https://soroban.stellar.org/docs/examples)
5. Open an issue on the project repository

## 📝 Notes

- Deployment typically takes 1-3 minutes
- Testnet deployments are free (after funding with Friendbot)
- Mainnet deployments require XLM for transaction fees
- Contract IDs are deterministic based on deployer and WASM hash
- Keep deployment logs for audit and troubleshooting purposes
