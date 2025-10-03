# Certificate Management Contract Deployment - Test Plan

## Test Objectives
Verify that the deployment script successfully builds, uploads, and deploys the certificate-management-contract to Stellar Testnet.

## Prerequisites Verification

### 1. Stellar CLI Installation
```bash
# Install Stellar CLI
cargo install stellar-cli

# Verify installation
stellar --version
```

### 2. Identity Setup
```bash
# Generate test identity
stellar keys generate test-deployer --network testnet

# Verify identity exists
stellar keys ls
```

### 3. Contract Structure Verification
- Verify Cargo.toml exists
- Verify src/lib.rs exists
- Verify contract builds successfully

## Test Cases

### Test Case 1: Script Help
**Command**: `./scripts/deploy_certificate_management_contract.zsh --help`
**Expected Result**: Display usage information
**Status**: ⏳ Pending

### Test Case 2: Missing Network Parameter
**Command**: `./scripts/deploy_certificate_management_contract.zsh`
**Expected Result**: Error message about missing network parameter
**Status**: ⏳ Pending

### Test Case 3: Invalid Network Parameter
**Command**: `./scripts/deploy_certificate_management_contract.zsh invalid`
**Expected Result**: Error message about invalid network
**Status**: ⏳ Pending

### Test Case 4: Build Contract
**Component**: `build_contract()` function
**Expected Result**: 
- Contract builds successfully
- WASM file created in target directory
- WASM file size displayed
**Status**: ⏳ Pending

### Test Case 5: Full Deployment to Testnet
**Command**: `./scripts/deploy_certificate_management_contract.zsh testnet test-deployer`
**Expected Results**:
1. ✅ Prerequisites check passes
2. ✅ Parameters validated
3. ✅ Contract builds successfully
4. ✅ WASM uploaded to testnet
5. ✅ WASM hash captured (64-char hex)
6. ✅ Contract deployed successfully
7. ✅ Contract ID captured (C + 55 chars)
8. ✅ Deployment results saved to JSON
9. ✅ Deployment summary created
10. ✅ Deployment log created
**Status**: ⏳ Pending

### Test Case 6: Verify Output Files
**Files to Check**:
- `ContractsRevo/certificate-management-contract/logs/deployment_YYYYMMDD_HHMMSS.log`
- `ContractsRevo/certificate-management-contract/logs/deployment_results.json`
- `ContractsRevo/certificate-management-contract/logs/latest_deployment.txt`

**Validation**:
- Files exist
- JSON is valid
- All required fields present
- Timestamps are correct
**Status**: ⏳ Pending

### Test Case 7: Contract Initialization
**Command**: 
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account test-deployer \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```
**Expected Result**: Contract initialized successfully
**Status**: ⏳ Pending

### Test Case 8: Contract Verification on Explorer
**Steps**:
1. Open https://testnet.stellar.org/explorer
2. Search for contract ID
3. Verify contract exists
**Expected Result**: Contract visible on explorer
**Status**: ⏳ Pending

## Script Validation Checklist

### Code Quality
- [x] Script follows project conventions
- [x] Uses same structure as other deployment scripts
- [x] Proper error handling
- [x] Colored output for user feedback
- [x] Comprehensive logging

### Functionality
- [x] Builds contract using `stellar contract build`
- [x] Uploads WASM using `stellar contract upload`
- [x] Deploys contract using `stellar contract deploy`
- [x] Handles network selection (testnet/mainnet)
- [x] Parses and displays wasm_hash
- [x] Parses and displays contract_id
- [x] Saves deployment logs
- [x] Creates JSON results file
- [x] Creates summary file

### Documentation
- [x] Usage documentation in script
- [x] Comprehensive DEPLOYMENT.md created
- [x] Examples provided
- [x] Troubleshooting guide included
- [x] Post-deployment steps documented

### Error Handling
- [x] Checks prerequisites
- [x] Validates parameters
- [x] Validates WASM file existence
- [x] Validates WASM hash format
- [x] Validates contract ID format
- [x] Provides meaningful error messages
- [x] Cleanup on error

## Test Execution Steps

### Step 1: Prepare Environment
```bash
# Navigate to project root
cd /home/jayy4rl/Revo-Contracts

# Ensure script is executable
chmod +x scripts/deploy_certificate_management_contract.zsh

# Install Stellar CLI if not already installed
cargo install stellar-cli
```

### Step 2: Setup Test Identity
```bash
# Generate test identity for testnet
stellar keys generate test-cert-deployer --network testnet

# Fund the account (testnet only)
# Visit https://laboratory.stellar.org/#account-creator?network=test
```

### Step 3: Run Help Command
```bash
./scripts/deploy_certificate_management_contract.zsh --help
```

### Step 4: Test Parameter Validation
```bash
# Test missing parameters
./scripts/deploy_certificate_management_contract.zsh

# Test invalid network
./scripts/deploy_certificate_management_contract.zsh invalid
```

### Step 5: Build Only Test
```bash
# Test build step (may need to modify script temporarily)
cd ContractsRevo/certificate-management-contract
stellar contract build --profile release
```

### Step 6: Full Deployment Test
```bash
# Run full deployment to testnet
./scripts/deploy_certificate_management_contract.zsh testnet test-cert-deployer
```

### Step 7: Verify Output Files
```bash
# Check log directory
ls -la ContractsRevo/certificate-management-contract/logs/

# View deployment results
cat ContractsRevo/certificate-management-contract/logs/deployment_results.json

# View deployment summary
cat ContractsRevo/certificate-management-contract/logs/latest_deployment.txt
```

### Step 8: Verify on Stellar Explorer
```bash
# Get contract ID from results
CONTRACT_ID=$(jq -r '.contract_id' ContractsRevo/certificate-management-contract/logs/deployment_results.json)

# Print contract ID
echo "Contract ID: $CONTRACT_ID"

# Open explorer (manual step)
echo "Verify at: https://testnet.stellar.org/explorer"
```

### Step 9: Initialize Contract
```bash
# Get contract ID and admin address
CONTRACT_ID=$(jq -r '.contract_id' ContractsRevo/certificate-management-contract/logs/deployment_results.json)
ADMIN_ADDRESS=$(stellar keys address test-cert-deployer)

# Initialize contract
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account test-cert-deployer \
  --network testnet \
  -- initialize \
  --admin $ADMIN_ADDRESS
```

### Step 10: Test Contract Functionality
```bash
# Issue a test certificate
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account test-cert-deployer \
  --network testnet \
  -- issue_certification \
  --issuer $ADMIN_ADDRESS \
  --recipient $ADMIN_ADDRESS \
  --cert_type "Organic" \
  --doc_hash "0000000000000000000000000000000000000000000000000000000000000001" \
  --expiry_timestamp 1735689600
```

## Success Criteria

### Required for Acceptance
- ✅ Script is executable
- ✅ Script located in `scripts/` directory
- ✅ Script builds contract using `stellar contract build`
- ✅ Script uploads WASM and captures wasm_hash
- ✅ Script deploys contract and captures contract_id
- ✅ Script supports network parameter (testnet/mainnet)
- ✅ Script handles errors with meaningful output
- ✅ Script saves deployment results to logs
- ✅ Usage documentation added (DEPLOYMENT.md)
- ⏳ Script tested successfully on Stellar Testnet

## Test Results

### Environment
- **OS**: Linux
- **Shell**: zsh
- **Date**: 2025-10-03
- **Tester**: [To be filled]

### Test Execution
[To be filled during actual testing]

### Issues Found
[To be documented during testing]

### Resolution
[To be documented during testing]

## Sign-off

- [ ] All test cases passed
- [ ] Documentation complete
- [ ] Script ready for production use
- [ ] Pull request created

---

**Next Steps After Testing**:
1. Document any issues found
2. Make necessary fixes
3. Re-test
4. Create remote branch
5. Push code
6. Create pull request to close issue #230
