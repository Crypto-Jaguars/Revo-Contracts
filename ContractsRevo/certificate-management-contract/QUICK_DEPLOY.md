# Certificate Management Contract - Quick Deploy Guide

## 🚀 Quick Start (5 Minutes)

### Step 1: Install Stellar CLI
```bash
cargo install stellar-cli
```

### Step 2: Create Testnet Identity
```bash
stellar keys generate cert-deployer --network testnet
```

### Step 3: Deploy Contract
```bash
cd /home/jayy4rl/Revo-Contracts
./scripts/deploy_certificate_management_contract.zsh testnet cert-deployer
```

### Step 4: Get Contract ID
```bash
cat ContractsRevo/certificate-management-contract/logs/deployment_results.json | grep contract_id
```

### Step 5: Initialize Contract
```bash
# Get admin address
ADMIN=$(stellar keys address cert-deployer)

# Get contract ID from deployment results
CONTRACT_ID=$(jq -r '.contract_id' ContractsRevo/certificate-management-contract/logs/deployment_results.json)

# Initialize
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account cert-deployer \
  --network testnet \
  -- initialize \
  --admin $ADMIN
```

## ✅ Done! Your contract is deployed and initialized.

---

## 📋 What Just Happened?

1. ✅ Built certificate management contract WASM
2. ✅ Uploaded WASM to Stellar Testnet
3. ✅ Deployed contract and got Contract ID
4. ✅ Saved deployment results to logs
5. ✅ Initialized contract with admin

## 🧪 Test Your Contract

### Issue a Certificate
```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source-account cert-deployer \
  --network testnet \
  -- issue_certification \
  --issuer $ADMIN \
  --recipient $ADMIN \
  --cert_type "Organic" \
  --doc_hash "0000000000000000000000000000000000000000000000000000000000000001" \
  --expiry_timestamp 1735689600
```

### Verify a Certificate
```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- verify_certification \
  --doc_hash "0000000000000000000000000000000000000000000000000000000000000001"
```

### View Certificate Details
```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_cert_by_doc_hash \
  --doc_hash "0000000000000000000000000000000000000000000000000000000000000001"
```

## 🌐 Verify on Explorer

Visit: https://testnet.stellar.org/explorer

Search for your Contract ID: `$CONTRACT_ID`

## 📁 Deployment Files

All deployment information is saved in:
```
ContractsRevo/certificate-management-contract/logs/
├── deployment_YYYYMMDD_HHMMSS.log  # Full log
├── deployment_results.json         # JSON results  
└── latest_deployment.txt           # Quick summary
```

## 🔄 Redeploy or Update

To redeploy (new contract instance):
```bash
./scripts/deploy_certificate_management_contract.zsh testnet cert-deployer
```

## 📖 Full Documentation

- **Deployment Guide**: [DEPLOYMENT.md](./DEPLOYMENT.md)
- **Test Plan**: [TEST_PLAN.md](./TEST_PLAN.md)
- **Contract README**: [README.md](./README.md)
- **Scripts README**: [../scripts/README.md](../scripts/README.md)

## ❓ Need Help?

1. Check deployment logs: `tail -f ContractsRevo/certificate-management-contract/logs/deployment_*.log`
2. Verify prerequisites: `./scripts/deploy_certificate_management_contract.zsh --help`
3. View test plan: `cat ContractsRevo/certificate-management-contract/TEST_PLAN.md`

## 🎯 Next Steps

1. ✅ Deploy to testnet ← You are here
2. ⏳ Test all contract functions
3. ⏳ Verify on Stellar Explorer
4. ⏳ Deploy to mainnet (when ready)

---

**Happy Deploying! 🚀**
