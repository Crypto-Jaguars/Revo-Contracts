# Farmer Insurance Contract Deployment Scripts

This directory contains automated deployment scripts for the Farmer Insurance Contract.

## Available Scripts

### 1. deploy_farmer_insurance.zsh (Unix/Linux/macOS)
Bash/Zsh script for Unix-based systems.

### 2. deploy_farmer_insurance.ps1 (Windows)
PowerShell script for Windows systems.

## Usage

### PowerShell (Windows)
```powershell
# Navigate to scripts directory
cd scripts

# Make script executable (if needed)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Deploy to testnet with default profile using alice identity
.\deploy_farmer_insurance.ps1 testnet default alice

# Deploy to mainnet with release profile using production key
.\deploy_farmer_insurance.ps1 mainnet release production_key
```

### Bash/Zsh (Unix/Linux/macOS)
```bash
# Navigate to scripts directory
cd scripts

# Make script executable
chmod +x deploy_farmer_insurance.zsh

# Deploy to testnet with default profile using alice identity
./deploy_farmer_insurance.zsh testnet default alice

# Deploy to mainnet with release profile using production key
./deploy_farmer_insurance.zsh mainnet release production_key
```

## Parameters

| Parameter | Description | Valid Values | Default |
|-----------|-------------|--------------|---------|
| network | Target network for deployment | testnet, mainnet | testnet |
| profile | Build profile to use | default, release, etc. | default |
| source_identity | Stellar CLI identity name | Any configured identity | alice |

## Script Features

- **Automated Build**: Builds the contract using `stellar contract build`
- **WASM Upload**: Uploads the compiled WASM and captures wasm_hash
- **Contract Deployment**: Deploys contract and captures contract_id
- **Error Handling**: Comprehensive error handling with meaningful messages
- **Logging**: Detailed logs saved to `logs/deployment_YYYYMMDD_HHMMSS.log`
- **JSON Summary**: Deployment summary saved as JSON for automation
- **Verification**: Attempts to verify deployment success
- **Color Output**: Colored console output for better readability

## Prerequisites

### Required Software
1. **Stellar CLI**: Install using `cargo install --locked stellar-cli`
2. **PowerShell** (Windows) or **Bash/Zsh** (Unix)

### Optional Software
- **jq**: For enhanced JSON parsing (Unix script only)

### Stellar CLI Setup
1. Generate or import an identity:
   ```bash
   stellar keys generate alice --network testnet
   ```

2. Fund your account (for testnet):
   ```bash
   stellar keys fund alice --network testnet
   ```

## Output Files

### Log Files
- **Location**: `ContractsRevo/farmer-insurance-contract/logs/`
- **Format**: `deployment_YYYYMMDD_HHMMSS.log`
- **Content**: Detailed deployment process logs with timestamps

### Summary Files
- **Location**: `ContractsRevo/farmer-insurance-contract/logs/`
- **Format**: `deployment_summary_YYYYMMDD_HHMMSS.json`
- **Content**: JSON summary with deployment results

### Example Summary JSON
```json
{
  "contract_name": "farmer-insurance-contract",
  "network": "testnet",
  "profile": "default",
  "source_identity": "alice",
  "deployment_timestamp": "2025-09-30 14:30:45 UTC",
  "wasm_hash": "b02c13e30dc02dd4d773f447b0e74d1d5fd08dad45309ad3419f024d1ed464bc",
  "contract_id": "CD4DEPWDUP4UQLQUF4WMXUOEXPBCKVY5CX7XTD2BDI6YWYHZQZARMGJP",
  "wasm_path": "./target/wasm32v1-none/release/farmer_insurance_contract.wasm",
  "wasm_size_bytes": 125648,
  "log_file": "./logs/deployment_20250930_143045.log"
}
```

## Error Handling

The scripts include comprehensive error handling for:
- Missing prerequisites (Stellar CLI, contract directory)
- Build failures
- Upload failures
- Deployment failures
- Invalid parameters
- Network connectivity issues

## Security Considerations

### Testnet vs Mainnet
- **Testnet**: Safe for testing, uses test tokens
- **Mainnet**: Real network with real tokens, use with caution

### Identity Management
- Store mainnet keys securely
- Never commit private keys to version control
- Use separate identities for testnet and mainnet

## Integration with CI/CD

The scripts are designed to be CI/CD friendly:
- Return appropriate exit codes (0 for success, 1 for failure)
- Generate machine-readable JSON summaries
- Provide detailed logging for debugging

### Example GitHub Actions Usage
```yaml
- name: Deploy Contract
  run: |
    ./scripts/deploy_farmer_insurance.zsh testnet default ${{ secrets.STELLAR_IDENTITY }}
  env:
    STELLAR_ACCOUNT: ${{ secrets.STELLAR_ACCOUNT }}
```

## Troubleshooting

### Common Issues

1. **"Stellar CLI not found"**
   - Install Stellar CLI: `cargo install --locked stellar-cli`
   - Ensure it's in your PATH

2. **"Identity not found"**
   - Create identity: `stellar keys generate <name> --network <network>`
   - Fund testnet account: `stellar keys fund <name> --network testnet`

3. **"Contract build failed"**
   - Check Rust installation and version
   - Verify contract source code compiles
   - Check build profile exists

4. **"Upload/Deploy failed"**
   - Check network connectivity
   - Verify account has sufficient funds
   - Check identity has proper permissions

### Getting Help
- Check the log files in the `logs/` directory
- Review the Stellar CLI documentation
- Verify prerequisites are met

## Example Usage Session

```powershell
PS> .\deploy_farmer_insurance.ps1 testnet default alice

================================
Farmer Insurance Contract Deployment
================================
[INFO] Starting deployment process...
[INFO] Checking prerequisites...
[SUCCESS] Stellar CLI found
[SUCCESS] Prerequisites check passed

================================
Building Contract
================================
[INFO] Building farmer-insurance-contract with profile: default
[SUCCESS] Contract built successfully
[INFO] WASM file created: ./target/wasm32v1-none/release/farmer_insurance_contract.wasm (125648 bytes)

================================
Uploading Contract
================================
[INFO] Uploading WASM to testnet network...
[SUCCESS] Contract uploaded successfully
[INFO] WASM Hash: b02c13e30dc02dd4d773f447b0e74d1d5fd08dad45309ad3419f024d1ed464bc

================================
Deploying Contract
================================
[INFO] Deploying contract to testnet network...
[SUCCESS] Contract deployed successfully
[INFO] Contract ID: CD4DEPWDUP4UQLQUF4WMXUOEXPBCKVY5CX7XTD2BDI6YWYHZQZARMGJP

================================
Deployment Summary
================================
[SUCCESS] Deployment summary saved to: ./logs/deployment_summary_20250930_143045.json

[INFO] === DEPLOYMENT RESULTS ===
[INFO] Contract Name: farmer-insurance-contract
[INFO] Network: testnet
[INFO] Profile: default
[INFO] Source Identity: alice
[INFO] WASM Hash: b02c13e30dc02dd4d773f447b0e74d1d5fd08dad45309ad3419f024d1ed464bc
[INFO] Contract ID: CD4DEPWDUP4UQLQUF4WMXUOEXPBCKVY5CX7XTD2BDI6YWYHZQZARMGJP
[INFO] Deployment Time: 2025-09-30 14:30:45 UTC
[INFO] Log File: ./logs/deployment_20250930_143045.log
[INFO] Summary File: ./logs/deployment_summary_20250930_143045.json

================================
Verifying Deployment
================================
[INFO] Verifying contract deployment...
[SUCCESS] Contract verification successful - contract is accessible

[SUCCESS] Deployment completed successfully!
```