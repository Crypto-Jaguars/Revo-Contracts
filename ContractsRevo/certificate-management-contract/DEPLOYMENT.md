# Certificate Management Contract Deployment Guide

## Overview
This guide provides detailed instructions for building, uploading, and deploying the Certificate Management Contract to Stellar networks (Testnet or Mainnet).

## Prerequisites

### Required Tools
1. **Stellar CLI**: Install the latest version
   ```bash
   cargo install stellar-cli
   ```

2. **Rust & Cargo**: Ensure you have Rust installed
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **jq** (Optional but recommended): For JSON parsing
   ```bash
   # Ubuntu/Debian
   sudo apt-get install jq
   
   # macOS
   brew install jq
   ```

### Stellar Identity Setup
Before deploying, you need a Stellar identity:

```bash
# Generate a new identity for testnet
stellar keys generate my-identity --network testnet

# Or use an existing identity
stellar keys ls
```

## Deployment Script

### Location
```
scripts/deploy_certificate_management_contract.zsh
```

### Usage

#### Basic Syntax
```bash
./scripts/deploy_certificate_management_contract.zsh [network] [identity]
```

#### Parameters
- **network** (required): Target network - `testnet` or `mainnet`
- **identity** (optional): Stellar identity/key to use (defaults to `default`)

### Examples

#### 1. Deploy to Testnet with Default Identity
```bash
./scripts/deploy_certificate_management_contract.zsh testnet
```

#### 2. Deploy to Testnet with Named Identity
```bash
./scripts/deploy_certificate_management_contract.zsh testnet alice
```

#### 3. Deploy to Mainnet with Production Identity
```bash
./scripts/deploy_certificate_management_contract.zsh mainnet production
```

#### 4. Display Help
```bash
./scripts/deploy_certificate_management_contract.zsh --help
```

## Deployment Process

The script executes the following steps automatically:

### 1. Prerequisites Check
- Verifies Stellar CLI is installed
- Verifies Cargo is installed
- Checks for jq (optional)

### 2. Parameter Validation
- Validates network parameter (testnet/mainnet)
- Validates identity exists and is configured
- Sets up logging directory

### 3. Contract Build
```bash
stellar contract build --profile release
```
- Builds the contract in release mode
- Locates WASM file in multiple possible locations
- Displays WASM file size

### 4. Contract Upload
```bash
stellar contract upload --source-account <profile> --network <network> --wasm <wasm_path>
```
- Uploads WASM to the specified network
- Captures and validates WASM hash (64-character hex string)
- Logs upload details

### 5. Contract Deployment
```bash
stellar contract deploy --source-account <profile> --network <network> --wasm-hash <wasm_hash>
```
- Deploys contract using the uploaded WASM hash
- Captures and validates Contract ID (starts with 'C', 56 characters)
- Logs deployment details

### 6. Results Storage
- Saves deployment results to JSON file
- Creates human-readable summary
- Generates timestamped deployment log

## Output Files

After successful deployment, the following files are created:

### 1. Deployment Log
**Location**: `ContractsRevo/certificate-management-contract/logs/deployment_YYYYMMDD_HHMMSS.log`

Contains detailed timestamped logs of the entire deployment process.

### 2. Deployment Results (JSON)
**Location**: `ContractsRevo/certificate-management-contract/logs/deployment_results.json`

```json
{
  "contract_name": "certificate-management-contract",
  "network": "testnet",
  "profile": "default",
  "wasm_hash": "a1b2c3d4...",
  "contract_id": "CABC123...",
  "deployment_timestamp": "2025-10-03 12:34:56 UTC",
  "wasm_path": "/path/to/wasm",
  "deployment_log": "/path/to/log"
}
```

### 3. Deployment Summary
**Location**: `ContractsRevo/certificate-management-contract/logs/latest_deployment.txt`

Human-readable summary of the latest deployment.

## Post-Deployment Steps

### 1. Initialize the Contract
After deployment, initialize the contract with an admin address:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <PROFILE> \
  --network <NETWORK> \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

### 2. Verify on Stellar Explorer

**Testnet Explorer**: https://testnet.stellar.org/explorer
**Mainnet Explorer**: https://stellar.org/explorer

Search for your contract ID to verify deployment.

### 3. Test Contract Functionality

Example: Issue a certificate
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <ISSUER> \
  --network testnet \
  -- issue_certification \
  --issuer <ISSUER_ADDRESS> \
  --recipient <RECIPIENT_ADDRESS> \
  --cert_type "Organic" \
  --doc_hash <DOCUMENT_HASH> \
  --expiry_timestamp <TIMESTAMP>
```

Example: Verify a certificate
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- verify_certification \
  --doc_hash <DOCUMENT_HASH>
```

## Troubleshooting

### Error: "Stellar CLI is not installed"
**Solution**: Install Stellar CLI
```bash
cargo install stellar-cli
```

### Error: "Identity does not exist"
**Solution**: Create or import an identity
```bash
stellar keys generate <name> --network <network>
```

### Error: "WASM file not found"
**Solution**: Ensure the contract builds successfully. Check:
- Cargo.toml is properly configured
- All dependencies are available
- Run `cargo clean` and rebuild

### Error: "Failed to extract WASM hash"
**Solution**: 
- Verify network connectivity
- Check Stellar CLI version
- Ensure sufficient funds in source account

### Error: "Contract deployment failed"
**Solution**:
- Verify WASM hash is correct
- Ensure source account has sufficient funds
- Check network status

## Script Features

### ✅ Automatic WASM Path Detection
The script automatically searches for WASM files in multiple locations:
- Modern Stellar CLI: `target/wasm32v1-none/release/`
- Legacy location: `target/wasm32-unknown-unknown/release/`
- Contract-specific build directory

### ✅ Comprehensive Error Handling
- Validates all prerequisites
- Checks parameters before execution
- Provides meaningful error messages
- Logs all errors with timestamps

### ✅ Colored Output
- **BLUE**: Informational messages
- **GREEN**: Success messages
- **YELLOW**: Warnings
- **RED**: Errors
- **CYAN**: Headers

### ✅ Detailed Logging
- Timestamped logs for audit trail
- Separate log file for each deployment
- JSON and text format results

### ✅ Network Support
- Testnet deployment for testing
- Mainnet deployment for production
- Network-specific instructions and links

## Security Best Practices

1. **Testnet First**: Always test on testnet before mainnet deployment
2. **Identity Management**: Keep production identities secure
3. **Verify Deployment**: Always verify on Stellar Explorer
4. **Backup Results**: Save deployment results and contract IDs
5. **Audit Logs**: Review deployment logs for any issues

## Additional Resources

- [Stellar Documentation](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/developer-tools/cli)
- [Certificate Management Contract README](../ContractsRevo/certificate-management-contract/README.md)

## Support

For issues or questions:
1. Check the deployment logs in `logs/` directory
2. Review this documentation
3. Open an issue on the project repository
4. Consult Stellar/Soroban documentation

## License

This deployment script is part of the Revo-Contracts project. See [LICENSE](../LICENSE) for details.
