# Contract Deployments

This document tracks all deployed contracts across different networks.

## Microlending Contract

### Testnet Deployments

| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| TBD | testnet | TBD | pending | Microlending contract deployment pending |


## Product Auction Contract

### Testnet Deployments

| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| CBORFMBO3CWISM5POWHFQIE2O2JAAGPCFVSRCTA2GL4KZJQIYN7NBGDI | testnet | 2025-09-28 | done | Product auction contract |


## Purchase Review Contract

### Testnet Deployments
| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| CBR7Z27SC7K3AJ7HZEZTFEQ7QJRNQNFF4FI3RBWIEZFTT3E7RHEYQPFL | testnet | 2025-09-28 | done | Purchase Review Contract |


## Transaction NFT Contract

### Testnet Deployments
| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| CBJJP6QKZOO53PKWPBZGJD533DL6W63YONCAU4LXIJ2TZ2MYEQL55M3F | testnet | 2025-09-28 | done | Transaction NFT Contract |


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







