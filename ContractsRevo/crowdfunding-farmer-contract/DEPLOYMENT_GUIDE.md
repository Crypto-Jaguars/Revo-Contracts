# Crowdfunding Farmer Contract Deployment Guide

## Overview

The `deploy_crowdfunding_farmer.zsh` script automates the complete deployment process of the crowdfunding farmer contract to Stellar networks (testnet or mainnet).

## Prerequisites

Before running the deployment script, ensure you have:

1. **Stellar CLI** installed:
   ```bash
   cargo install stellar-cli
   ```

2. **Rust and Cargo** installed (for building the contract)

3. **Valid Stellar identity** configured:
   ```bash
   # Generate a new identity for testnet
   stellar keys generate alice --network testnet
   
   # Or for mainnet
   stellar keys generate production --network mainnet
   ```

4. **Optional: jq** for JSON parsing (recommended):
   - Linux: `sudo apt-get install jq`
   - macOS: `brew install jq`
   - Windows: Download from https://jqlang.github.io/jq/

## Usage

```bash
./deploy_crowdfunding_farmer.zsh [network] [identity]
```

### Parameters

- **network** (required): Target network (`testnet` or `mainnet`)
- **identity** (optional): Stellar identity to use (defaults to `default`)

### Examples

```bash
# Deploy to testnet with default identity
./deploy_crowdfunding_farmer.zsh testnet

# Deploy to testnet with specific identity
./deploy_crowdfunding_farmer.zsh testnet alice

# Deploy to mainnet with production identity
./deploy_crowdfunding_farmer.zsh mainnet production

# Show help
./deploy_crowdfunding_farmer.zsh --help
```

## What the Script Does

The deployment script performs the following steps:

1. **Prerequisites Check**: Verifies required tools are installed
2. **Parameter Validation**: Checks network and identity parameters
3. **Contract Build**: Builds the contract using `stellar contract build`
4. **Contract Upload**: Uploads WASM to Stellar network and captures WASM hash
5. **Contract Deploy**: Deploys contract using WASM hash and captures contract ID
6. **Results Saving**: Saves deployment logs and results in JSON format
7. **Results Display**: Shows deployment summary with next steps

## Output Files

The script creates several files in the `ContractsRevo/crowdfunding-farmer-contract/logs/` directory:

- **`deployment_YYYYMMDD_HHMMSS.log`**: Detailed deployment log
- **`deployment_results.json`**: Machine-readable deployment results
- **`latest_deployment.txt`**: Human-readable deployment summary

### Sample deployment_results.json

```json
{
  "contract_name": "crowdfunding-farmer-contract",
  "network": "testnet",
  "profile": "alice",
  "wasm_hash": "a1b2c3d4e5f6...",
  "contract_id": "CABCD1234567890...",
  "deployment_timestamp": "2025-10-02 15:30:45 UTC",
  "wasm_path": "/path/to/crowdfunding_farmer_contract.wasm",
  "deployment_log": "/path/to/deployment_log.log"
}
```

## Contract Features

The crowdfunding farmer contract includes:

- **Campaign Management**: Create and manage farmer crowdfunding campaigns
- **Contribution Handling**: Accept and track contributions from supporters
- **Reward Distribution**: Distribute rewards to contributors based on campaign success
- **Status Tracking**: Monitor campaign progress and completion status

## Troubleshooting

### Common Issues

1. **"Stellar CLI not found"**
   - Install Stellar CLI: `cargo install stellar-cli`

2. **"Identity does not exist"**
   - Generate identity: `stellar keys generate <name> --network <network>`
   - List identities: `stellar keys ls`

3. **"WASM file not found"**
   - The script will automatically search multiple locations
   - Check that the contract builds successfully
   - Verify `stellar contract build` completes without errors

4. **"Upload/Deploy failed"**
   - Check network connectivity
   - Verify identity has sufficient XLM for fees
   - For testnet, get test XLM from friendbot

### Getting Test XLM (Testnet Only)

```bash
# Get test XLM for your identity
stellar keys address alice  # Get your address
# Then visit: https://friendbot.stellar.org/?addr=YOUR_ADDRESS
```

## Next Steps After Deployment

1. **Verify on Explorer**: Check deployment on Stellar Explorer
   - Testnet: https://testnet.stellar.org/explorer
   - Mainnet: https://stellar.org/explorer

2. **Test Contract**: Invoke contract methods to verify functionality

3. **Initialize Contract**: Set up initial contract state if required

4. **Create Campaigns**: Start creating crowdfunding campaigns for farmers

## Security Considerations

- **Mainnet Deployments**: Use secure, well-funded identities for production
- **Key Management**: Store production keys securely
- **Testing**: Thoroughly test on testnet before mainnet deployment
- **Code Review**: Ensure contract code is audited and secure

## Support

For issues or questions:
- Check the deployment logs for detailed error information
- Verify all prerequisites are properly installed
- Test deployment on testnet first
- Review Stellar CLI documentation for network-specific requirements