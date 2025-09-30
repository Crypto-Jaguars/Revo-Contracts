# Crop Yield Prediction Contract

A Stellar smart contract for predicting crop yields based on various agricultural factors.

## Features

- Crop yield prediction algorithms
- Data collection and reporting
- Historical data analysis
- Prediction accuracy tracking

## Contract Functions

The contract provides functions for:
- Submitting crop data (`register_crop`)
- Generating yield predictions (`generate_prediction`)
- Retrieving prediction history (`get_prediction`)
- Managing prediction models (`update_data_source`)
- Listing predictions by crop (`list_predictions_by_crop`)
- Listing predictions by region (`list_predictions_by_region`)

## Deployment

### Testnet Deployment (Test Only)

**Status**: Test deployment only - not for production use

| Field | Value |
|-------|-------|
| Network | Stellar Testnet |
| Contract ID | CD4DEPWDUP4UQLQUF4WMXUOEXPBCKVY5CX7XTD2BDI6YWYHZQZARMGJP |
| Deployment Date | 2025-09-30 |
| Deployer | GDCLDPFHDMNBYBCDIWAF4EAT6YQLCSPN2LRQOCND3675F27QCWQGLCGI |
| Transaction Hash | 7abf5076b9ba0eedb6f2f166651ed1b3be7d1799282dd2bebe2fcc0dd24a3641 |
| Explorer Link | https://stellar.expert/explorer/testnet/contract/CD4DEPWDUP4UQLQUF4WMXUOEXPBCKVY5CX7XTD2BDI6YWYHZQZARMGJP |

### Build Instructions

```bash
# Navigate to contract directory
cd ContractsRevo/crop-yield-prediction

# Build the contract
stellar contract build
```

### Deployment Commands

```bash
# Upload contract
stellar contract upload \
  --source alice \
  --network testnet \
  --wasm ./target/wasm32v1-none/release/crop_yield_prediction.wasm

# Deploy contract
stellar contract deploy \
  --source alice \
  --network testnet \
  --wasm ./target/wasm32v1-none/release/crop_yield_prediction.wasm
```

## Testing

This contract is currently deployed for testing purposes only. Do not use in production environments.

### Verify Deployment

You can verify the deployment was successful by checking the contract on Stellar Expert:
https://stellar.expert/explorer/testnet/contract/CD4DEPWDUP4UQLQUF4WMXUOEXPBCKVY5CX7XTD2BDI6YWYHZQZARMGJP

## License

This project is licensed under the terms specified in the repository root.