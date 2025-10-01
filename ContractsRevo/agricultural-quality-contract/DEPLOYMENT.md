# Agricultural Quality Contract - Deployment

## Location

```
scripts/deploy_agricultural_quality.zsh
```

## Prerequisites

- Stellar CLI (v21.0.0+)
- Rust & Cargo
- Stellar identity with funded account

## Setup

```bash
# Generate and fund identity (testnet)
stellar keys generate testnet_account --network testnet
stellar keys fund testnet_account --network testnet
```

## Usage

```bash
cd scripts
./deploy_agricultural_quality.zsh [network] [identity]
```

- `network`: `testnet` or `mainnet`
- `identity`: Stellar identity name (defaults to `default`)

### Examples

```bash
./deploy_agricultural_quality.zsh testnet testnet_account
./deploy_agricultural_quality.zsh testnet
./deploy_agricultural_quality.zsh --help
```

## What it does

1. Builds the contract via `stellar contract build`
2. Uploads WASM and captures `wasm_hash`
3. Deploys contract and captures `contract_id`
4. Saves logs to `ContractsRevo/agricultural-quality-contract/logs/`

## Output Files

- `deployment_YYYYMMDD_HHMMSS.log`
- `deployment_results.json`
- `latest_deployment.txt`

## Verify

Testnet Explorer: https://testnet.stellar.org/explorer

Search with your Contract ID from `latest_deployment.txt`.
