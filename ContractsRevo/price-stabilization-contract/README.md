# Price Stabilization Contract

## 🎯 Overview

The Price Stabilization Contract is a decentralized smart contract built on the Soroban framework for the Stellar blockchain. It enables agricultural cooperatives and market participants to mitigate price volatility for agricultural products by managing a stabilization fund and distributing payments to farmers when prices fall below a predefined threshold.

This contract helps protect farmers from market price fluctuations while ensuring sustainable agricultural practices and market stability.

## 🛠 Contract Functionality

### **1. Stabilization Fund Management**

- Create and manage multiple stabilization funds for different crop types
- Track contributions from farmers, buyers, and other stakeholders
- Maintain secure fund balances with proper access controls
- Adjust price thresholds based on market conditions

### **2. Price Monitoring**

- Integrate with off-chain oracles to fetch real-time market prices
- Chainlink Oracle Integration** - Decentralized price feeds with high reliability
- Define and update price thresholds for triggering distributions
- Monitor price movements and identify when thresholds are crossed
- Maintain historical price data for analysis

### **3. Payout Distribution**

- Automatically calculate payout amounts based on price differences
- Distribute funds to eligible farmers when prices fall below thresholds
- Track payout history and fund utilization
- Ensure fair distribution based on production capacity

## 🔗 Chainlink Integration

### **Chainlink Oracle Features**

- **Decentralized Price Feeds**: Multiple oracle nodes provide price data
- **High Reliability**: Redundant data sources prevent single points of failure
- **Real-time Updates**: Automated price feed updates
- **Tamper-resistant**: Cryptographic proof of data integrity
- **Multi-source Aggregation**: Reduces manipulation risk

### **Chainlink Functions**

```rust
// Register a Chainlink price feed for a crop type
fn register_chainlink_feed(
    env: Env,
    admin: Address,
    crop_type: String,
    feed_address: Address,
    decimals: u32,
    description: String,
) -> Result<(), StabilizationError>

// Get current price from Chainlink feed
fn get_chainlink_price(
    env: Env,
    crop_type: String,
) -> Result<(i128, u64), StabilizationError>

// Update price from Chainlink feed with validation
fn update_chainlink_price(
    env: Env,
    oracle: Address,
    crop_type: String,
    price: i128,
    timestamp: u64,
    round_id: u64,
    decimals: u32,
) -> Result<(), StabilizationError>
```

### **Chainlink Data Validation**

- **Staleness Check**: Rejects data older than 1 hour
- **Round ID Validation**: Ensures sequential price updates
- **Price Validation**: Confirms positive price values
- **Decimal Conversion**: Standardizes price format
- **Authorization**: Only registered feeds can update prices

## 📦 Key Data Structures

```rust
struct StabilizationFund {
    fund_id: BytesN<32>,
    fund_name: String,
    admin: Address,
    total_balance: i128,
    price_threshold: i128,
    crop_type: String,
    active: bool,
    creation_time: u64,
    last_payout_time: Option<u64>,
}

struct Payout {
    farmer_id: Address,
    fund_id: BytesN<32>,
    amount: i128,
    timestamp: u64,
    market_price: i128,
    threshold_price: i128,
}

// NEW: Chainlink Integration Structures
struct ChainlinkPriceFeed {
    feed_address: Address,
    decimals: u32,
    description: String,
    crop_type: String,
    registered_time: u64,
    active: bool,
}

struct ChainlinkPriceData {
    price: i128,
    timestamp: u64,
    feed_address: Address,
    round_id: u64,
    decimals: u32,
}
```

## 🚀 Setup Guide

### **Prerequisites**

Ensure you have the following installed:

- Rust & Cargo
- Soroban CLI
- Stellar Standalone/Testnet/Mainnet access
- Node.js (for interacting with the contract via scripts)

### **Installation Steps**

1. **Clone the Repository**

   ```bash
   git clone https://github.com/Crypto-Jaguars/Revo-Contracts.git
   cd ContractsRevo/price-stabilization-contract
   ```

2. **Build the Contract**

   ```bash
   stellar contract build
   ```

3. **Run the Tests**

   ```bash
   cargo test
   ```

4. **Deploy to Testnet**
   ```bash
   make deploy-testnet ADMIN_SECRET=your_secret_key
   ```

## 📝 Usage Examples

### **Creating a Stabilization Fund**

```javascript
// Using JavaScript SDK
const fundName = "Corn Stabilization Fund";
const priceThreshold = 1000; // $10.00 with 2 decimal places
const cropType = "corn";

const fundId = await contract.call(
  "create_fund",
  adminPublicKey,
  fundName,
  priceThreshold,
  cropType
);
```

### **Contributing to a Fund**

```javascript
const amount = 5000; // $50.00

await contract.call("contribute_fund", contributorPublicKey, fundId, amount);
```

### **Triggering a Payout**

```javascript
const farmers = [farmer1PublicKey, farmer2PublicKey];

const distribution = await contract.call(
  "trigger_payout",
  adminPublicKey,
  fundId,
  farmers
);
```

### **Chainlink Integration Examples**

```javascript
// Register a Chainlink price feed
await contract.call(
  "register_chainlink_feed",
  adminPublicKey,
  "corn",
  chainlinkFeedAddress,
  8, // decimals
  "Corn Price Feed"
);

// Get price from Chainlink feed
const [price, timestamp] = await contract.call("get_chainlink_price", "corn");

// Update price from Chainlink oracle
await contract.call(
  "update_chainlink_price",
  oraclePublicKey,
  "corn",
  1050, // price in cents
  currentTimestamp,
  12345, // round_id
  8 // decimals
);
```

## 🔒 Security Considerations

- The contract implements strict access controls to prevent unauthorized fund withdrawals
- Only registered oracles can update price data
- **Chainlink feeds require proper authorization and validation**
- Fund administrators must be authenticated for sensitive operations
- Payouts are only triggered when prices fall below thresholds
- **Chainlink data includes staleness checks and round ID validation**

## 🔄 Integration Points

- **Oracle Integration**: The contract can integrate with external price oracles to fetch real-time market prices
- **Chainlink Integration**: Decentralized price feeds with high reliability and tamper-resistance
- **Microlending Contract**: Can be integrated with the microlending contract for emergency loans when payouts are insufficient
- **Cooperative Management**: Works with the cooperative management contract for governance decisions

## 📚 References

- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Chainlink Documentation](https://docs.chain.link/)

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
