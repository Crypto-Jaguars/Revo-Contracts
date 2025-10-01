# Farmer Staking Contract

A comprehensive smart contract for staking farmer tokens on the Stellar network. This contract enables farmers to stake their tokens to earn rewards, promoting long-term holding and providing liquidity incentives within the Revo ecosystem.

## Features

- **Flexible Staking Pools**: Create and manage multiple staking pools with customizable parameters
- **Lock Period Rewards**: Higher rewards for longer lock periods to incentivize long-term participation
- **Reward Compounding**: Automatically reinvest rewards to maximize returns
- **Emergency Unstaking**: Early withdrawal option with penalty for urgent situations
- **Pool Management**: Admin controls for pausing, reward rate updates, and pool configuration
- **Low Fee Optimization**: Designed for minimal transaction costs on Stellar
- **Scalability**: Support for multiple pools with different agricultural reward structures

## Contract Structure

```
farmer-staking-contract/
├── src/
│   ├── lib.rs           # Main contract interface and exports
│   ├── pool.rs          # Staking pool management logic
│   ├── staking.rs       # Core staking and unstaking logic
│   ├── rewards.rs       # Reward calculation and distribution
│   ├── utils.rs         # Shared utilities and token integration
│   └── tests/           # Comprehensive unit tests
├── Cargo.toml           # Rust dependencies and configuration
├── Makefile            # Build and deployment automation
└── README.md           # This file
```

## Key Data Structures

### RewardPool
```rust
pub struct RewardPool {
    pub pool_id: BytesN<32>,
    pub admin: Address,
    pub token_address: Address,
    pub total_staked: i128,
    pub reward_rate: i128,
    pub current_epoch: u64,
    pub min_stake_amount: i128,
    pub max_lock_period: u64,
    pub is_paused: bool,
    pub created_at: u64,
    pub last_reward_update: u64,
}
```

### Stake
```rust
pub struct Stake {
    pub farmer_id: Address,
    pub pool_id: BytesN<32>,
    pub amount: i128,
    pub stake_time: u64,
    pub lock_period: u64,
    pub unlock_time: u64,
    pub reward_debt: i128,
}
```

## Core Functions

### Pool Management

#### `initialize_pool`
Initialize a new staking pool with reward settings.

```rust
pub fn initialize_pool(
    env: Env,
    admin: Address,
    token_address: Address,
    reward_rate: i128,
    min_stake_amount: i128,
    max_lock_period: u64,
) -> Result<BytesN<32>, PoolError>
```

**Parameters:**
- `admin`: Address that will manage the pool
- `token_address`: Address of the farmer token contract
- `reward_rate`: Rewards per epoch (in token units)
- `min_stake_amount`: Minimum amount required to stake
- `max_lock_period`: Maximum lock period in seconds

**Returns:** Unique pool identifier

#### `update_reward_rate`
Update the reward rate for a pool (admin only).

```rust
pub fn update_reward_rate(
    env: Env,
    admin: Address,
    pool_id: BytesN<32>,
    new_reward_rate: i128,
) -> Result<(), PoolError>
```

#### `pause_pool` / `unpause_pool`
Pause or unpause staking in a pool (admin only).

### Staking Operations

#### `stake`
Stake farmer tokens with an optional lock period.

```rust
pub fn stake(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
    amount: i128,
    lock_period: u64,
) -> Result<(), StakeError>
```

**Parameters:**
- `farmer`: Address of the farmer staking tokens
- `pool_id`: Pool to stake into
- `amount`: Amount of tokens to stake
- `lock_period`: Duration in seconds to lock tokens (0 for no lock)

**Lock Period Bonus Multipliers:**
- No lock: 100% (base)
- < 1 week: 105% (5% bonus)
- < 1 month: 110% (10% bonus)
- < 3 months: 120% (20% bonus)
- < 6 months: 135% (35% bonus)
- < 1 year: 150% (50% bonus)
- >= 1 year: 175% (75% bonus)

#### `unstake`
Unstake tokens and claim accumulated rewards after lock period.

```rust
pub fn unstake(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
    amount: i128,
) -> Result<(), StakeError>
```

#### `emergency_unstake`
Emergency unstaking with 10% penalty for early withdrawal.

```rust
pub fn emergency_unstake(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
    amount: i128,
) -> Result<i128, StakeError>
```

**Returns:** Amount after penalty

### Reward Functions

#### `claim_rewards`
Claim pending rewards without unstaking.

```rust
pub fn claim_rewards(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
) -> Result<i128, RewardError>
```

**Returns:** Amount of rewards claimed

#### `compound_rewards`
Compound rewards by restaking them.

```rust
pub fn compound_rewards(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
) -> Result<i128, RewardError>
```

**Returns:** Amount of rewards compounded

### Query Functions

#### `get_stake_info`
Query stake details and pending rewards for a farmer.

```rust
pub fn get_stake_info(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
) -> Result<(Stake, i128), StakeError>
```

**Returns:** Tuple of (Stake info, pending rewards)

#### `get_pool_info`
Get pool information.

```rust
pub fn get_pool_info(
    env: Env,
    pool_id: BytesN<32>
) -> Result<RewardPool, PoolError>
```

#### `get_all_pools`
Get all active pool IDs.

```rust
pub fn get_all_pools(env: Env) -> Vec<BytesN<32>>
```

#### `get_total_staked`
Get total value locked in a pool.

```rust
pub fn get_total_staked(
    env: Env,
    pool_id: BytesN<32>
) -> Result<i128, PoolError>
```

## Building and Testing

### Prerequisites

- Rust 1.70 or higher
- Stellar CLI
- wasm32-unknown-unknown target

### Install Dependencies

```bash
make install-deps
```

### Build the Contract

```bash
make build
```

### Run Tests

```bash
# Run all tests
make test

# Run tests with output
make test-verbose

# Run a specific test
make test-single TEST_NAME=test_name
```

### Format and Lint

```bash
# Format code
make fmt

# Run linter
make lint

# Run all checks
make check
```

## Deployment

### Deploy to Testnet

```bash
SOURCE_ACCOUNT=your_account_here make deploy-testnet
```

### Deploy to Mainnet

```bash
SOURCE_ACCOUNT=your_account_here make deploy-mainnet
```

### Optimize WASM

```bash
make optimize
```

## Usage Example

### 1. Initialize a Staking Pool

```rust
let pool_id = contract.initialize_pool(
    &env,
    &admin,
    &token_address,
    1000,           // reward_rate: 1000 tokens per epoch
    100,            // min_stake: 100 tokens minimum
    31536000,       // max_lock: 1 year maximum
);
```

### 2. Stake Tokens

```rust
// Approve tokens first
token_contract.approve(&farmer, &staking_contract, 1000);

// Stake with 6 month lock period for 35% bonus
contract.stake(
    &env,
    &farmer,
    &pool_id,
    1000,           // amount
    15552000,       // lock_period: 6 months
);
```

### 3. Check Stake Info

```rust
let (stake, pending_rewards) = contract.get_stake_info(
    &env,
    &farmer,
    &pool_id,
);
```

### 4. Claim Rewards

```rust
let claimed = contract.claim_rewards(&env, &farmer, &pool_id);
```

### 5. Compound Rewards

```rust
let compounded = contract.compound_rewards(&env, &farmer, &pool_id);
```

### 6. Unstake

```rust
// After lock period expires
contract.unstake(&env, &farmer, &pool_id, 1000);
```

## Reward Calculation

Rewards are calculated using the following formula:

```
user_share = (staked_amount / total_staked) * precision
epochs_passed = time_staked / epoch_duration
base_rewards = (reward_rate * user_share * epochs_passed) / precision
final_rewards = (base_rewards * lock_multiplier) / 100 - reward_debt
```

Where:
- `epoch_duration` = 86400 seconds (1 day)
- `precision` = 1,000,000 (for accurate calculations)
- `lock_multiplier` = bonus based on lock period (100-175)
- `reward_debt` = previously claimed rewards

## Security Features

1. **Authentication**: All user functions require caller authentication
2. **Admin Controls**: Sensitive operations restricted to pool admin
3. **Overflow Protection**: Safe arithmetic operations throughout
4. **Lock Period Enforcement**: Cannot unstake before lock period expires
5. **Penalty Mechanism**: 10% penalty for emergency unstaking
6. **Pool Pausing**: Admin can pause pool during emergencies
7. **Validation**: Comprehensive input validation

## Integration with Farmer Token Contract

This contract integrates directly with the [farmer-token-contract](../farmer-token-contract) for secure token transfers. Users must approve the staking contract to transfer tokens on their behalf.

## Future Enhancements

- Integration with price-stabilization-contract for dynamic reward adjustments
- Multiple token reward pools
- NFT-based staking certificates
- Governance voting power based on staked amount
- Delegation of staking positions
- Advanced slashing mechanisms for cooperative rule violations

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please read the [CONTRIBUTING](../../CONTRIBUTING.ms) guidelines before submitting PRs.

## Support

For issues and questions:
- Open an issue on GitHub
- Check existing documentation in [docs/](../../docs/)
- Review the [Soroban Documentation](https://soroban.stellar.org/docs)

## References

- [Stellar Official Guide](https://developers.stellar.org/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Soroban Examples](https://github.com/stellar/soroban-examples)
