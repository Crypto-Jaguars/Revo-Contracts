# Deployment Scripts

This directory contains automated deployment scripts for Revo Contracts.

## Available Deployment Scripts

- [`deploy_water_management.zsh`](#water-management-contract-deployment) - Water Management Contract
- [`deploy_product_auction.zsh`](#product-auction-contract-deployment) - Product Auction Contract
- [`deploy_crop_yield_prediction.zsh`](#crop-yield-prediction-contract-deployment) - Crop Yield Prediction Contract
- [`deploy_agricultural_quality.zsh`](#agricultural-quality-contract-deployment) - Agricultural Quality Contract
- [`deploy_transaction_nft_contract.zsh`](#transaction-nft-contract-deployment) - Transaction NFT Contract
 feat/deploy-the-crowdfunding-farmer-contract
 feat/deploy-the-crowdfunding-farmer-contract
- [`deploy_crowdfunding_farmer.zsh`](#crowdfunding-farmer-contract-deployment) - Crowdfunding Farmer Contract

feat/equipment-rental-contract
- [`deploy_crowdfunding_farmer.zsh`](#crowdfunding-farmer-contract-deployment) - Crowdfunding Farmer Contract
- [`deploy_equipment_rental.zsh`](#equipment-rental-contract-deployment) - Equipment Rental Contract

 main
- [`deploy_certificate_management_contract.zsh`](#certificate-management-contract-deployment) - Certificate Management Contract
 main


## Transaction NFT Contract Deployment

The `deploy_transaction_nft_contract.zsh` script automates the complete deployment process for the transaction NFT contract to Stellar networks.

### Features

- ✅ Automated build, upload, and deploy process
- ✅ Network selection (testnet/mainnet)
- ✅ Identity-based deployment
- ✅ Comprehensive error handling and validation
- ✅ Detailed logging with timestamps
- ✅ JSON and text result files
- ✅ Colored output for better readability
- ✅ Prerequisites validation

### Usage

```bash
./deploy_transaction_nft_contract.zsh [network] [identity]
```

**Parameters:**
- `network` (required): `testnet` or `mainnet`
- `identity` (optional): Stellar identity name (defaults to `default`)

**Examples:**
```bash
# Deploy to testnet with default identity
./deploy_transaction_nft_contract.zsh testnet

# Deploy to testnet with specific identity
./deploy_transaction_nft_contract.zsh testnet alice

# Deploy to mainnet with production identity
./deploy_transaction_nft_contract.zsh mainnet production

# View help
./deploy_transaction_nft_contract.zsh --help
```

### Contract Functions

The deployed contract provides the following functions:
- `mint_nft(buyer, seller, amount, product)` - Mint transaction NFT
- `get_nft_metadata(tx_id)` - Retrieve NFT metadata

### Output Files

The script creates several files in `ContractsRevo/transaction-nft-contract/logs/`:

- `deployment_YYYYMMDD_HHMMSS.log` - Detailed deployment log
- `deployment_results.json` - JSON file with deployment metadata
- `latest_deployment.txt` - Human-readable summary

---

## Crop Yield Prediction Contract Deployment

The `deploy_crop_yield_prediction.zsh` script automates building, uploading, and deploying the crop yield prediction contract to Stellar networks.

### Usage

```bash
./deploy_crop_yield_prediction.zsh [network] [identity]
```

**Parameters:**
- `network` (required): `testnet` or `mainnet`
- `identity` (optional): Stellar identity name (defaults to `default`)

**Examples:**
```bash
# Deploy to testnet
./deploy_crop_yield_prediction.zsh testnet testnet_account

# View help
./deploy_crop_yield_prediction.zsh --help
```

### Documentation

See `ContractsRevo/crop-yield-prediction/DEPLOYMENT.md` for complete usage guide.

---

## Agricultural Quality Contract Deployment

The `deploy_agricultural_quality.zsh` script automates building, uploading, and deploying the agricultural quality contract to Stellar networks.

### Usage

```bash
./deploy_agricultural_quality.zsh [network] [identity]
```

**Parameters:**
- `network` (required): `testnet` or `mainnet`
- `identity` (optional): Stellar identity name (defaults to `default`)

**Examples:**
```bash
# Deploy to testnet
./deploy_agricultural_quality.zsh testnet testnet_account

# View help
./deploy_agricultural_quality.zsh --help
```

### Documentation

See `ContractsRevo/agricultural-quality-contract/DEPLOYMENT.md` for the complete usage guide.

---

## Crowdfunding Farmer Contract Deployment

The `deploy_crowdfunding_farmer.zsh` script automates the complete deployment process for the crowdfunding farmer contract to Stellar networks.

### Features

- ✅ Automated build, upload, and deploy process
- ✅ Network selection (testnet/mainnet)
- ✅ Identity-based deployment
- ✅ Comprehensive error handling and validation
- ✅ Detailed logging with timestamps
- ✅ JSON and text result files
- ✅ Colored output for better readability
- ✅ Prerequisites validation

### Usage

```bash
./deploy_crowdfunding_farmer.zsh [network] [identity]
```

**Parameters:**
- `network` (required): `testnet` or `mainnet`
- `identity` (optional): Stellar identity name (defaults to `default`)

**Examples:**
```bash
# Deploy to testnet with default identity
./deploy_crowdfunding_farmer.zsh testnet

# Deploy to testnet with specific identity
./deploy_crowdfunding_farmer.zsh testnet alice

# Deploy to mainnet with production identity
./deploy_crowdfunding_farmer.zsh mainnet production

# View help
./deploy_crowdfunding_farmer.zsh --help
```

### Contract Functions

The deployed contract provides the following functions:
- `create_campaign(farmer, goal, duration, description)` - Create new crowdfunding campaign
- `contribute(campaign_id, amount)` - Contribute to a campaign
- `distribute_rewards(campaign_id)` - Distribute rewards to contributors
- `get_campaign(campaign_id)` - Retrieve campaign information
- `get_contributions(campaign_id)` - Get campaign contributions

### Output Files

The script creates several files in `ContractsRevo/crowdfunding-farmer-contract/logs/`:

- `deployment_YYYYMMDD_HHMMSS.log` - Detailed deployment log
- `deployment_results.json` - JSON file with deployment metadata
- `latest_deployment.txt` - Human-readable summary

### Documentation

See `ContractsRevo/crowdfunding-farmer-contract/DEPLOYMENT_GUIDE.md` for the complete usage guide.

---

 feat/deploy-the-crowdfunding-farmer-contract

## Equipment Rental Contract Deployment

The `deploy_equipment_rental.zsh` script automates the complete deployment process for the equipment rental contract to Stellar networks.

### Features

- ✅ Automated build, upload, and deploy process
- ✅ Network selection (testnet/mainnet)
- ✅ Identity-based deployment
- ✅ Comprehensive error handling and validation
- ✅ Detailed logging with timestamps
- ✅ JSON and text result files
- ✅ Colored output for better readability
- ✅ Prerequisites validation
- ✅ Cross-platform support (Windows PowerShell wrapper included)

### Usage

```bash
./deploy_equipment_rental.zsh [network] [identity]
```

**Parameters:**
- `network` (required): `testnet` or `mainnet`
- `identity` (optional): Stellar identity name (defaults to `default`)

**Examples:**
```bash
# Deploy to testnet with default identity
./deploy_equipment_rental.zsh testnet

# Deploy to testnet with specific identity
./deploy_equipment_rental.zsh testnet alice

# Deploy to mainnet with production identity
./deploy_equipment_rental.zsh mainnet production

# View help
./deploy_equipment_rental.zsh --help
```

### Windows Support

**PowerShell Wrapper:**
```powershell
.\deploy_equipment_rental.ps1 testnet alice
```

**Manual PowerShell Script:**
```powershell
.\deploy_equipment_rental_manual.ps1 -Network testnet -Identity alice
```

### Contract Functions

The deployed contract provides the following functions:
- `register_equipment` - Register new equipment to the platform
- `update_availability` - Change equipment availability status
- `rent_equipment` - Rent equipment for specified duration
- `return_equipment` - Return rented equipment
- `schedule_maintenance` - Schedule equipment maintenance
- `update_maintenance_status` - Update maintenance completion
- `get_equipment_details` - Retrieve equipment information
- `get_rental_history` - Get equipment rental history

### Output Files

The script creates several files in `ContractsRevo/equipment-rental-contract/logs/`:

- `deployment_YYYYMMDD_HHMMSS.log` - Detailed deployment log
- `deployment_results.json` - JSON file with deployment metadata
- `latest_deployment.txt` - Human-readable summary

### Documentation

See `ContractsRevo/equipment-rental-contract/DEPLOYMENT_GUIDE.md` for the complete usage guide.

---

 main
## Water Management Contract Deployment

The `deploy_water_management.zsh` script automates the complete deployment process for the water management contract to Stellar networks.

### Features

- ✅ Automated build, upload, and deploy process
- ✅ Network selection (testnet/mainnet)
- ✅ Profile-based deployment
- ✅ Comprehensive error handling and validation
- ✅ Detailed logging with timestamps
- ✅ JSON and text result files
- ✅ Colored output for better readability
- ✅ Prerequisites validation

### Prerequisites

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

### Usage

```bash
./deploy_water_management.zsh [network] [profile]
```

#### Parameters

- `network` (required): Target network - `testnet` or `mainnet`
- `profile` (optional): Stellar profile to use (defaults to `default`)

#### Examples

```bash
# Deploy to testnet with default profile
./deploy_water_management.zsh testnet

# Deploy to testnet with custom profile
./deploy_water_management.zsh testnet my_testnet_profile

# Deploy to mainnet with production profile
./deploy_water_management.zsh mainnet production

# Show help
./deploy_water_management.zsh --help
```

### What the Script Does

1. **Prerequisites Check**: Validates that all required tools are installed
2. **Parameter Validation**: Ensures network and profile parameters are valid
3. **Contract Build**: Uses `stellar contract build` to compile the contract
4. **Contract Upload**: Uploads the WASM file and captures the WASM hash
5. **Contract Deploy**: Deploys the contract and captures the contract ID
6. **Result Saving**: Saves deployment results in multiple formats
7. **Logging**: Creates detailed logs with timestamps

### Output Files

The script creates several files in the contract's `logs/` directory:

- `deployment_YYYYMMDD_HHMMSS.log` - Detailed deployment log
- `deployment_results.json` - JSON file with deployment metadata
- `latest_deployment.txt` - Human-readable summary

### Example Output

```
================================
BUILDING CONTRACT
================================
[INFO] Building contract with profile: default
[SUCCESS] Contract built successfully
[INFO] WASM file: /path/to/water_management_contract.wasm
[INFO] WASM size: 2.1M

================================
UPLOADING CONTRACT
================================
[INFO] Uploading contract to testnet network
[SUCCESS] Contract uploaded successfully
[INFO] WASM Hash: CC3B2F8A9D7E6F4A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8A9B0C1D2E3F4

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
  Contract: water-management-contract
  Network: testnet
  Profile: default
  WASM Hash: CC3B2F8A9D7E6F4A1B2C3D4E5F6A7B8C9D0E1F2A3B4C5D6E7F8A9B0C1D2E3F4
  Contract ID: CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN

Files Created:
  Deployment Log: /path/to/logs/deployment_20241201_143022.log
  Results JSON: /path/to/logs/deployment_results.json
  Summary: /path/to/logs/latest_deployment.txt
```

### Error Handling

The script includes comprehensive error handling:

- **Prerequisites validation**: Checks for required tools
- **Parameter validation**: Validates network and profile parameters
- **Build validation**: Ensures contract builds successfully
- **Upload validation**: Verifies WASM upload and hash extraction
- **Deploy validation**: Confirms successful deployment and contract ID extraction
- **Exit codes**: Proper exit codes for automation and CI/CD integration

### Troubleshooting

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
./deploy_water_management.zsh testnet
```

### Security Considerations

- **Testnet First**: Always test deployments on testnet before mainnet
- **Profile Security**: Use secure profiles with appropriate permissions
- **Log Files**: Deployment logs may contain sensitive information
- **Network Validation**: Script validates network parameter to prevent accidents

### Integration

The script is designed for easy integration with:

- **CI/CD Pipelines**: Proper exit codes and structured output
- **Automation Tools**: JSON output for programmatic processing
- **Monitoring**: Detailed logging for audit trails
- **Documentation**: Automatic result file generation

### Support

For issues or questions:

1. Check the deployment logs for detailed error information
2. Verify all prerequisites are installed and configured
3. Ensure network connectivity and account funding
4. Review Stellar CLI documentation for network-specific requirements

---

## Product Auction Contract Deployment

The `deploy_product_auction.zsh` script automates the deployment of the product auction contract. It follows the same pattern as other deployment scripts with all the standard features.

### Usage

```bash
./deploy_product_auction.zsh [network] [profile]
```

See the Water Management section above for detailed documentation on features, prerequisites, and usage patterns. All deployment scripts share the same core functionality.

---

## General Prerequisites

All deployment scripts require:

1. **Stellar CLI**: `cargo install stellar-cli`
2. **jq**: `brew install jq` (macOS) or `apt-get install jq` (Linux)
3. **Rust & Cargo**: https://rustup.rs/
4. **Stellar Profile**: Configured with funded account

## Common Usage Patterns

### View Help
```bash
./deploy_[contract_name].zsh --help
```

### Deploy to Testnet
```bash
./deploy_[contract_name].zsh testnet
```

### Deploy to Mainnet
```bash
./deploy_[contract_name].zsh mainnet production_profile
```

## Best Practices

1. **Always test on testnet first** before deploying to mainnet
2. **Use separate profiles** for different environments
3. **Review logs** after deployment
4. **Backup deployment results** including WASM hash and Contract ID
5. **Verify on Explorer** after deployment

## Certificate Management Contract Deployment

The `deploy_certificate_management_contract.zsh` script automates the complete deployment process for the certificate management contract to Stellar networks.

### Features

- ✅ Automated build, upload, and deploy process
- ✅ Network selection (testnet/mainnet)
- ✅ Identity-based deployment
- ✅ Comprehensive error handling and validation
- ✅ Detailed logging with timestamps
- ✅ JSON and text result files
- ✅ Colored output for better readability
- ✅ Prerequisites validation
- ✅ WASM path auto-detection across multiple locations

### Usage

```bash
./deploy_certificate_management_contract.zsh [network] [identity]
```

**Parameters:**
- `network` (required): `testnet` or `mainnet`
- `identity` (optional): Stellar identity name (defaults to `default`)

**Examples:**
```bash
# Deploy to testnet with default identity
./deploy_certificate_management_contract.zsh testnet

# Deploy to testnet with specific identity
./deploy_certificate_management_contract.zsh testnet alice

# Deploy to mainnet with production identity
./deploy_certificate_management_contract.zsh mainnet production

# View help
./deploy_certificate_management_contract.zsh --help
```

### Prerequisites

1. **Stellar CLI**: Install with `cargo install stellar-cli`
2. **Rust & Cargo**: Required for building contracts
3. **Stellar Identity**: Create with `stellar keys generate <name> --network <network>`
4. **jq** (optional): For enhanced JSON parsing

### What the Script Does

1. **Prerequisites Check**: Validates required tools are installed
2. **Parameter Validation**: Ensures network and identity are valid
3. **Contract Build**: Builds using `stellar contract build --profile release`
4. **WASM Upload**: Uploads to specified network and captures WASM hash
5. **Contract Deploy**: Deploys using WASM hash and captures Contract ID
6. **Results Storage**: Saves deployment results to JSON and text files
7. **Logging**: Creates timestamped deployment logs

### Output Files

After successful deployment:

```
ContractsRevo/certificate-management-contract/logs/
├── deployment_YYYYMMDD_HHMMSS.log      # Detailed deployment log
├── deployment_results.json              # JSON results file
└── latest_deployment.txt                # Human-readable summary
```

### Post-Deployment

After deployment, initialize the contract:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <IDENTITY> \
  --network <NETWORK> \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

### Documentation

For detailed deployment guide, see: [DEPLOYMENT.md](../ContractsRevo/certificate-management-contract/DEPLOYMENT.md)

### Troubleshooting

Common issues and solutions:

1. **WASM file not found**: Script auto-detects WASM in multiple locations
2. **Identity not found**: Run `stellar keys ls` to view available identities
3. **Network error**: Ensure network connectivity and try again
4. **Build fails**: Check Rust/Cargo installation and dependencies

## Resources

- [Stellar Smart Contracts Documentation](https://developers.stellar.org/docs/smart-contracts)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/cli)
- [Testnet Explorer](https://testnet.stellar.org/explorer)
- [Mainnet Explorer](https://stellar.org/explorer)
