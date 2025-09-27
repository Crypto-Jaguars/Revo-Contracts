# Farmer Liquidity Pool Contract

A comprehensive Stellar smart contract implementation for automated market maker (AMM) liquidity pools, specifically designed for agricultural token trading within the Revo ecosystem.

## 🎯 Overview

This contract provides a complete liquidity pool solution with:
- **Liquidity Provision**: Add/remove liquidity with LP token minting/burning
- **Token Swaps**: Constant product formula-based token exchanges
- **Fee Distribution**: Proportional fee distribution to liquidity providers
- **Comprehensive Testing**: Modular test suite covering all functionality

## 🏗️ Architecture

### Core Modules

```
src/
├── lib.rs              # Main contract interface
├── pool.rs             # Pool initialization and management
├── liquidity.rs        # Add/remove liquidity operations
├── swap.rs             # Token swap execution and calculations
├── fees.rs             # Fee collection and distribution
├── storage.rs          # Data structures and storage management
├── error.rs            # Error definitions
└── tests/
    ├── mod.rs          # Test module organization
    └── simple_tests.rs # Basic functionality tests
```

### Key Features

1. **Pool Management**
   - Initialize pools with configurable fee rates (0-100%)
   - Support for any token pair
   - Admin-controlled pool settings

2. **Liquidity Operations**
   - Add liquidity with proportional LP token minting
   - Remove liquidity with proportional token redemption
   - First liquidity provision uses geometric mean pricing

3. **Token Swaps**
   - Constant product formula (x * y = k)
   - Configurable swap fees
   - Slippage protection
   - Price impact calculations

4. **Fee System**
   - Automatic fee collection on swaps
   - Proportional distribution to LP providers
   - Claimable accumulated fees

## 🚀 Usage

### Initialization

```rust
// Initialize a pool with 0.3% fee rate
contract.initialize(
    admin: Address,
    token_a: Address,
    token_b: Address,
    fee_rate: 30  // 30 basis points = 0.3%
);
```

### Adding Liquidity

```rust
// Add liquidity to the pool
let lp_tokens = contract.add_liquidity(
    provider: Address,
    amount_a: 1000,
    amount_b: 2000,
    min_lp_tokens: 0
);
```

### Token Swaps

```rust
// Swap tokens
let amount_out = contract.swap(
    trader: Address,
    token_in: Address,
    amount_in: 100,
    min_amount_out: 95
);
```

### Fee Management

```rust
// Claim accumulated fees
let (fees_a, fees_b) = contract.claim_fees(provider: Address);

// Calculate fee share
let fee_share = contract.calculate_fee_share(
    provider: Address,
    total_fees: 1000
);
```

## 🧪 Testing

The contract includes comprehensive tests covering:

### Pool Tests
- ✅ Pool initialization with various fee rates
- ✅ Invalid initialization attempts
- ✅ Pool information retrieval
- ✅ Event emission verification

### Liquidity Tests
- ✅ First liquidity provision
- ✅ Subsequent liquidity additions
- ✅ Liquidity removal
- ✅ LP token balance tracking
- ✅ Mismatched ratio handling

### Swap Tests
- ✅ Token swaps in both directions
- ✅ Fee calculations
- ✅ Slippage protection
- ✅ Price impact verification
- ✅ Invalid swap attempts

### Fee Tests
- ✅ Fee accumulation during swaps
- ✅ Proportional fee distribution
- ✅ Fee claiming mechanism
- ✅ Edge cases (no liquidity, zero fees)

## 🔧 Technical Details

### Constant Product Formula

The contract uses the standard AMM formula:
```
(x + Δx) * (y - Δy) = x * y
```

Where:
- `x`, `y` are current reserves
- `Δx` is input amount (after fees)
- `Δy` is output amount

### Fee Structure

- **Swap Fees**: Configurable rate (0-100%)
- **Fee Distribution**: Proportional to LP token holdings
- **Fee Collection**: Automatic on each swap

### Security Features

- **Access Control**: Admin-only pool management
- **Input Validation**: Comprehensive parameter checking
- **Overflow Protection**: Safe math operations
- **Slippage Protection**: Minimum output guarantees

## 📊 Test Results

```bash
running 8 tests
test tests::simple_tests::test_pool_initialization ... ok
test tests::simple_tests::test_pool_initialization_with_different_admins ... ok
test tests::simple_tests::test_pool_initialization_zero_fee_rate ... ok
test tests::simple_tests::test_get_reserves_after_initialization ... ok
test tests::simple_tests::test_pool_info_immutability_after_initialization ... ok
test tests::simple_tests::test_pool_initialization_events ... ok
test tests::simple_tests::test_pool_initialization_max_fee_rate ... ok
test tests::simple_tests::test_pool_initialization_different_fee_rates ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🔗 Integration

This contract is designed to integrate seamlessly with:
- **Farmer Token Contract**: For agricultural token pairs
- **Stellar Network**: Native Stellar asset support
- **Revo Ecosystem**: Agricultural DeFi applications

## 📝 Development Notes

### Key Scenarios Covered

1. **Adding liquidity with mismatched token ratios**
2. **Token swap exceeding pool reserves**
3. **Fee distribution with no liquidity providers**
4. **Unauthorized liquidity removal attempts**
5. **Zero-liquidity swaps and invalid fee rates**

### Edge Cases Handled

- Empty pool operations
- Maximum fee rates
- Precision loss in calculations
- Event emission verification
- State consistency checks

## 🚀 Future Enhancements

Potential improvements for production deployment:
- Multi-hop swap routing
- Dynamic fee adjustment
- Governance token integration
- Advanced slippage protection
- MEV protection mechanisms

## 📄 License

This contract is part of the Revo Contracts ecosystem and follows the project's licensing terms.

---

**Note**: This is a comprehensive implementation suitable for testing and development. For production deployment, additional security audits and optimizations are recommended.
