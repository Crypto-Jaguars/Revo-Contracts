# Contract Deployment Script

This script allows you to deploy individual Stellar smart contracts to the testnet with custom source keys.

## Prerequisites

- Rust and Cargo installed
- Stellar CLI installed
- A funded Stellar account

## Setup

### Make the script executable

```bash
chmod +x ./scripts/deployer.sh   
```

### Setup (if you don't have a source account)

If you don't have a Stellar account set up, generate and fund one:

```bash
stellar keys generate <source_key_name> --network testnet --fund
```

This will create a new account and fund it with testnet XLM.

## Usage

```bash
./scripts/deployer.sh --source <source_key> --wasm <wasm_file_name>
```

### Arguments

- `--source`: Your Stellar source key name (e.g., `kennyv4`)
- `--wasm`: WASM file name without `.wasm` extension

### Examples

```bash
# Deploy product auction contract
./scripts/deployer.sh --source kennyv4 --wasm product_auction_contract

# Deploy loyalty token contract
./scripts/deployer.sh --source mykey --wasm loyalty_token_contract

# Show help and available contracts
./scripts/deployer.sh --help
```

## Available Contracts

- `product_auction_contract`
- `purchase_review_contract`
- `market_demand_forecasting_contract`
- `loyalty_token_contract`
- `land_leasing_contract`
- `farmer_insurance_contract`
- `environmental_impact_tracking`
- `csa_membership_contract`
- `crowdfunding_farmer_contract`
- `cross_cooperative_trade_contract`
- `supply_chain_tracking_contract`
- `micro_lending`
- `farmer_token_contract`
- `agricultural_auction_contract`
- `agricultural_quality_contract`
- `agricultural_training_contract`
- `certificate_management_contract`
- `commodity_token_contract`
- `crop_yield_prediction`
- `equipment_rental_contract`
- `farmer_liquidity_pool_contract`
- `rating_system_contract`
- `transaction_nft_contract`
- `water_management_contract`
- `a_lp_token_contract`

## What the Script Does

1. Checks and installs the `wasm32v1-none` Rust target if needed
2. Builds all contracts using `cargo build --target wasm32v1-none --release`
3. Uploads the specified WASM file to Stellar testnet
4. Deploys the contract with a generated alias
5. Returns the contract ID for future interactions

## Error Handling

The script includes proper error handling for:
- Missing arguments
- Non-existent WASM files
- Upload failures
- Deployment failures
- Empty contract IDs

## Notes

- All deployments are to Stellar testnet
- Contract aliases are auto-generated (e.g., `ProductAuctionContract`)
- The script will exit with error codes if any step fails
