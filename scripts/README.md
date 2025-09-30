# Deployment Scripts

This directory contains automated deployment scripts for Revo Contracts.

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
