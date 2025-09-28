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

## Market Demand Forecasting Contract

### Testnet Deployments
| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| CBAQQ2WHODYMR6W4AEPABCDVW4UHMGRMK6SESD6NZ4LZSE7HUVP6I77R | testnet | 2025-09-28 | done | Market Demand Forecasting Contract |

## Loyalty Token Contract

### Testnet Deployments
| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| CAW4JRV4WUGQ2EMRATLMMYBIA5IHTUMOD6I4LFZJI3BX445YS2ZFE46N | testnet | 2025-09-28 | done | Loyalty Token Contract |


## Land Leasing Contract

### Testnet Deployments
| Contract ID | Network | Deployed Date | Status | Notes |
|-------------|---------|---------------|--------|-------|
| CB7UOYOVLZWBWOHBBUM5RPZGX3XKHKWAWNA7DYIXHVR3TA4A7GLX22DU | testnet | 2025-09-28 | done | Land Leasing Contract |

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






