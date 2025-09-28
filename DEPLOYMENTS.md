# Contract Deployments

This document tracks all deployed contracts across different networks.

## Microlending Contract

### Testnet Deployments

| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| TBD | testnet | TBD | pending | Microlending contract deployment pending |

### Mainnet Deployments

| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| TBD | mainnet | TBD | pending | Not yet deployed to mainnet |

## Deployment Instructions

### Prerequisites
1. Install Soroban CLI: `cargo install soroban-cli`
2. Set up your identity: `soroban config identity generate <name>`
3. Fund your account for deployment fees

### Deploy to Testnet
```bash
cd ContractsRevo/microlending-contract
export ADMIN_SECRET=<your-secret-key>
make deploy-testnet
```

### Deploy to Mainnet
```bash
cd ContractsRevo/microlending-contract
export ADMIN_SECRET=<your-secret-key>
make deploy-mainnet
```

## Contract Verification

After deployment, verify the contract is working:

```bash
# Check contract info
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <your-identity> \
  --network testnet \
  -- \
  get_loan_request \
  --loan_id 1
```

## Notes

- All testnet deployments are for testing purposes only
- Mainnet deployments require careful consideration and testing
- Contract IDs should be updated in this file after successful deployment
