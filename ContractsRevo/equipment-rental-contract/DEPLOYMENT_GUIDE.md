# Equipment Rental Contract Deployment Guide

## Overview

The `deploy_equipment_rental.zsh` script automates the complete deployment process of the equipment rental contract to Stellar networks (testnet or mainnet).

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
./deploy_equipment_rental.zsh [network] [identity]
```

### Parameters

- **network** (required): Target network (`testnet` or `mainnet`)
- **identity** (optional): Stellar identity to use (defaults to `default`)

### Examples

```bash
# Deploy to testnet with default identity
./deploy_equipment_rental.zsh testnet

# Deploy to testnet with specific identity
./deploy_equipment_rental.zsh testnet alice

# Deploy to mainnet with production identity
./deploy_equipment_rental.zsh mainnet production

# Show help
./deploy_equipment_rental.zsh --help
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

The script creates several files in the `ContractsRevo/equipment-rental-contract/logs/` directory:

- **`deployment_YYYYMMDD_HHMMSS.log`**: Detailed deployment log
- **`deployment_results.json`**: Machine-readable deployment results
- **`latest_deployment.txt`**: Human-readable deployment summary

### Sample deployment_results.json

```json
{
  "contract_name": "equipment-rental-contract",
  "network": "testnet",
  "profile": "alice",
  "wasm_hash": "a1b2c3d4e5f6...",
  "contract_id": "CABCD1234567890...",
  "deployment_timestamp": "2025-10-02 15:30:45 UTC",
  "wasm_path": "/path/to/equipment_rental_contract.wasm",
  "deployment_log": "/path/to/deployment_log.log"
}
```

## Contract Features

The equipment rental contract includes:

- **Equipment Management**: Register and manage equipment inventory
- **Rental System**: Handle equipment rental bookings and returns
- **Maintenance Tracking**: Track equipment maintenance schedules and history
- **Pricing Management**: Set and update rental pricing per equipment type
- **Availability Control**: Manage equipment availability status

## Contract Functions

Once deployed, the contract provides these functions:

- `register_equipment` - Register new equipment to the platform
- `update_availability` - Change equipment availability status
- `rent_equipment` - Rent equipment for specified duration
- `return_equipment` - Return rented equipment
- `schedule_maintenance` - Schedule equipment maintenance
- `update_maintenance_status` - Update maintenance completion
- `get_equipment_details` - Retrieve equipment information
- `get_rental_history` - Get equipment rental history

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

## Windows Support

### PowerShell Wrapper
Use the PowerShell wrapper for Windows:
```powershell
# Deploy to testnet
.\scripts\deploy_equipment_rental.ps1 testnet

# Deploy to mainnet with specific identity
.\scripts\deploy_equipment_rental.ps1 mainnet production
```

### Manual PowerShell Script
For direct PowerShell execution:
```powershell
.\scripts\deploy_equipment_rental_manual.ps1 -Network testnet -Identity alice
```

## Next Steps After Deployment

1. **Verify on Explorer**: Check deployment on Stellar Explorer
   - Testnet: https://testnet.stellar.org/explorer
   - Mainnet: https://stellar.org/explorer

2. **Test Contract**: Invoke contract methods to verify functionality

3. **Register Equipment**: Add equipment to the rental platform

4. **Create Rental Listings**: Set up equipment for rental

5. **Test Workflows**: Test complete rental and maintenance cycles

## Security Considerations

- **Mainnet Deployments**: Use secure, well-funded identities for production
- **Key Management**: Store production keys securely
- **Testing**: Thoroughly test on testnet before mainnet deployment
- **Code Review**: Ensure contract code is audited and secure
- **Access Control**: Implement proper permission controls for equipment management

## Integration Examples

### Register Equipment
```bash
stellar contract invoke \
  --source-account alice \
  --network testnet \
  --id CONTRACT_ID \
  -- register_equipment \
  --id "equipment_001" \
  --equipment_type "Tractor" \
  --rental_price_per_day 100 \
  --location "Farm_A"
```

### Rent Equipment
```bash
stellar contract invoke \
  --source-account alice \
  --network testnet \
  --id CONTRACT_ID \
  -- rent_equipment \
  --equipment_id "equipment_001" \
  --renter "USER_ADDRESS" \
  --rental_days 7
```

## Support

For issues or questions:
- Check the deployment logs for detailed error information
- Verify all prerequisites are properly installed
- Test deployment on testnet first
- Review Stellar CLI documentation for network-specific requirements
- Check the equipment rental contract source code for function specifications