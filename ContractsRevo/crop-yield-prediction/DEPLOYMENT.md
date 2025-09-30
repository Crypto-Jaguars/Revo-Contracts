# Crop Yield Prediction - Deployment Script

## Overview

Automated script for building, uploading, and deploying the crop-yield-prediction contract to Stellar networks.

## Location

```
scripts/deploy_crop_yield_prediction.zsh
```

## Prerequisites

- Stellar CLI (v21.0.0+)
- Rust & Cargo
- Stellar identity with funded account

## Setup

### Create and Fund Identity

```bash
# Generate testnet identity
stellar keys generate testnet_account --network testnet

# Fund testnet account
stellar keys fund testnet_account --network testnet

# Verify identity exists
stellar keys ls
```

## Usage

```bash
cd scripts
./deploy_crop_yield_prediction.zsh [network] [identity]
```

### Parameters

- `network` (required): `testnet` or `mainnet`
- `identity` (optional): Stellar identity name (defaults to `default`)

### Examples

```bash
# Deploy to testnet with specific identity
./deploy_crop_yield_prediction.zsh testnet testnet_account

# Deploy to testnet with default identity
./deploy_crop_yield_prediction.zsh testnet

# View help
./deploy_crop_yield_prediction.zsh --help
```

## What the Script Does

1. **Validates** prerequisites and parameters
2. **Builds** the contract using `stellar contract build`
3. **Uploads** WASM and captures the WASM hash
4. **Deploys** the contract and captures the Contract ID
5. **Saves** deployment results to logs

## Output Files

After deployment, files are saved in `logs/`:

- `deployment_YYYYMMDD_HHMMSS.log` - Detailed deployment log
- `deployment_results.json` - Machine-readable results
- `latest_deployment.txt` - Human-readable summary

### Example JSON Output

```json
{
  "contract_name": "crop-yield-prediction",
  "network": "testnet",
  "profile": "testnet_account",
  "wasm_hash": "b5249824...",
  "contract_id": "CC7FMTUTE...",
  "deployment_timestamp": "2025-09-30 16:17:20 UTC",
  "wasm_path": "/path/to/crop_yield_prediction.wasm",
  "deployment_log": "/path/to/deployment_20250930_171712.log"
}
```

## Deployment Results

View your deployment summary:

```bash
cat ContractsRevo/crop-yield-prediction/logs/latest_deployment.txt
```

Get Contract ID:

```bash
cat ContractsRevo/crop-yield-prediction/logs/deployment_results.json | grep contract_id
```

## Verify Deployment

Visit Stellar Explorer:
- **Testnet**: https://testnet.stellar.org/explorer
- **Mainnet**: https://stellar.org/explorer

## Common Commands

```bash
# List identities
stellar keys ls

# View script help
./deploy_crop_yield_prediction.zsh --help

# Check deployment logs
cat ../ContractsRevo/crop-yield-prediction/logs/latest_deployment.txt
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Identity not found | Run `stellar keys generate <name> --network testnet` |
| Account not funded | Run `stellar keys fund <name> --network testnet` |
| Build fails | Verify Rust installation and contract dependencies |

## Script Features

- ✅ Automated build, upload, and deploy
- ✅ Network selection (testnet/mainnet)
- ✅ Identity-based authentication
- ✅ Error handling with meaningful messages
- ✅ Color-coded console output
- ✅ Multi-format logging (log, JSON, summary)
- ✅ WASM hash and Contract ID capture
- ✅ Compatible with Stellar CLI v23.x

---

**Last Tested**: September 30, 2025 on Stellar Testnet
**Status**: ✅ Working
