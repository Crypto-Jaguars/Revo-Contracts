# Farmer Token Contract

## Overview

The Farmer Token Contract is a SEP-10 compliant fungible token implementation on the Stellar blockchain using Soroban smart contracts. It's specifically designed for agricultural ecosystems, enabling farmers to hold and transfer tokens representing agricultural value, rewards, and achievements.

## Features

### Core Token Functionality
- **SEP-10 Compliant**: Implements standard fungible token interface
- **Transfer & Approve**: Standard ERC-20-like transfer and approval mechanisms
- **Minting**: Controlled token creation for rewarding farmers
- **Burning**: Token destruction for redemption or penalties

### Agricultural-Specific Features
- **Milestone-Based Minting**: Reward farmers for agricultural achievements
- **Batch Operations**: Efficient token distribution during harvest seasons
- **Role-Based Access Control**: Admin and minter role management
- **Pausable Transfers**: Emergency halt functionality for security

### Integration Points
- Compatible with agricultural quality contracts for quality-based rewards
- Supports cooperative management systems
- Ready for DeFi integrations (staking, yield farming)

## Contract Structure

```
farmer-token-contract/
├── src/
│   ├── lib.rs          # Main contract entry point
│   ├── token.rs        # Core token logic
│   ├── mint.rs         # Minting functionality
│   ├── burn.rs         # Burning functionality
│   ├── utils.rs        # Utilities and access control
│   └── test.rs         # Comprehensive test suite
├── Cargo.toml          # Dependencies
├── Makefile           # Build automation
└── README.md          # This file
```

## Prerequisites

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Stellar CLI**
   ```bash
   cargo install --locked stellar-cli --features opt
   ```

3. **WebAssembly target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Quick Start

### Building the Contract

```bash
# Using make
make build

# Or directly
stellar contract build
```

### Running Tests

```bash
# Run all tests
make test

# Run tests with output
make test-verbose

# Run specific test
make test-single TEST_NAME=test_mint_tokens
```

### Deployment

1. **Set up your account**
   ```bash
   export SOURCE_ACCOUNT="your_stellar_account_here"
   ```

2. **Deploy to testnet**
   ```bash
   make deploy-testnet
   ```

3. **Initialize the contract**
   ```bash
   stellar contract invoke \
     --id YOUR_CONTRACT_ID \
     --source $SOURCE_ACCOUNT \
     --network testnet \
     -- \
     initialize \
     --admin $SOURCE_ACCOUNT \
     --name "Farmer Token" \
     --symbol "FRM" \
     --decimals 7
   ```

## Usage Examples

### Initialize Token

```rust
let name = String::from_str(&env, "Farmer Token");
let symbol = String::from_str(&env, "FRM");
let decimals = 7u32;

client.initialize(&admin, &name, &symbol, &decimals);
```

### Minting Tokens

```bash
# Mint tokens to a farmer
stellar contract invoke \
  --id YOUR_CONTRACT_ID \
  --source $MINTER_ACCOUNT \
  --network testnet \
  -- \
  mint \
  --minter $MINTER_ACCOUNT \
  --to FARMER_ADDRESS \
  --amount 10000000000  # 1000 tokens with 7 decimals
```

### Transfer Tokens

```bash
# Transfer tokens between addresses
stellar contract invoke \
  --id YOUR_CONTRACT_ID \
  --source $FROM_ACCOUNT \
  --network testnet \
  -- \
  transfer \
  --from $FROM_ACCOUNT \
  --to RECIPIENT_ADDRESS \
  --amount 1000000000  # 100 tokens
```

### Approve Spending

```bash
# Approve another address to spend tokens
stellar contract invoke \
  --id YOUR_CONTRACT_ID \
  --source $OWNER_ACCOUNT \
  --network testnet \
  -- \
  approve \
  --owner $OWNER_ACCOUNT \
  --spender SPENDER_ADDRESS \
  --amount 5000000000  # 500 tokens
```

## Advanced Features

### Milestone-Based Minting

Reward farmers for achieving agricultural milestones:

```rust
let milestone = Symbol::new(&env, "harvest_complete");
client.mint_for_milestone(&minter, &farmer, &milestone, &amount);
```

### Batch Minting

Distribute tokens to multiple farmers efficiently:

```rust
let recipients = vec![
    &env,
    (farmer1, 1000_0000000),
    (farmer2, 2000_0000000),
    (farmer3, 3000_0000000),
];
client.batch_mint(&minter, &recipients);
```

### Token Redemption

Farmers can burn tokens for real-world value:

```rust
let redemption_type = Symbol::new(&env, "equipment");
client.burn_for_redemption(&farmer, &amount, &redemption_type);
```

### Admin Functions

```bash
# Add a new minter
stellar contract invoke \
  --id YOUR_CONTRACT_ID \
  --source $ADMIN_ACCOUNT \
  --network testnet \
  -- \
  add_minter \
  --admin $ADMIN_ACCOUNT \
  --minter NEW_MINTER_ADDRESS

# Pause transfers
stellar contract invoke \
  --id YOUR_CONTRACT_ID \
  --source $ADMIN_ACCOUNT \
  --network testnet \
  -- \
  pause \
  --admin $ADMIN_ACCOUNT
```

## Security Considerations

1. **Access Control**: Only authorized minters can create new tokens
2. **Pausable**: Admin can pause all transfers in case of emergency
3. **Balance Checks**: All operations verify sufficient balances
4. **Input Validation**: Amounts must be positive integers
5. **Event Logging**: All significant operations emit events for transparency

## Storage Optimization

The contract uses Stellar's storage efficiently:
- **Instance Storage**: Admin, metadata, paused state
- **Persistent Storage**: Balances, allowances, minters
- **Zero Balance Removal**: Automatically removes zero balances to save storage

## Integration with Agricultural Ecosystem

This token contract is designed to integrate with other Revo contracts:

1. **Quality Verification**: Mint tokens based on quality certifications
2. **Auction Rewards**: Distribute tokens for successful auctions
3. **Training Completion**: Award tokens for completing agricultural training
4. **Cooperative Rewards**: Enable cooperatives to distribute tokens to members

## Gas Optimization

- Batch operations reduce transaction costs
- Storage cleanup for zero balances
- Efficient data structures using Soroban SDK

## Testing

The contract includes comprehensive tests covering:
- Token initialization
- Minting and burning
- Transfers and approvals
- Access control
- Pause functionality
- Edge cases and error handling

Run the full test suite:
```bash
cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This contract is part of the Revo Contracts ecosystem. See the main repository for license information.

## Support

For questions and support:
- Create an issue in the GitHub repository
- Contact the development team
- Review the [Stellar documentation](https://soroban.stellar.org)

## Roadmap

- [ ] Integration with OpenZeppelin Stellar Contracts library
- [ ] Staking mechanism for yield farming
- [ ] Governance token functionality
- [ ] Cross-chain bridge support
- [ ] Advanced agricultural metrics integration