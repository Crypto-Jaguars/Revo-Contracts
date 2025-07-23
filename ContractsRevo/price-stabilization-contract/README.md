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
- Define and update price thresholds for triggering distributions
- Monitor price movements and identify when thresholds are crossed
- Maintain historical price data for analysis

### **3. Payout Distribution**

- Automatically calculate payout amounts based on price differences
- Distribute funds to eligible farmers when prices fall below thresholds
- Track payout history and fund utilization
- Ensure fair distribution based on production capacity

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

await contract.call(
  "contribute_fund",
  contributorPublicKey,
  fundId,
  amount
);
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

## 🔒 Security Considerations

- The contract implements strict access controls to prevent unauthorized fund withdrawals
- Only registered oracles can update price data
- Fund administrators must be authenticated for sensitive operations
- Payouts are only triggered when prices fall below thresholds

## 🔄 Integration Points

- **Oracle Integration**: The contract can integrate with external price oracles to fetch real-time market prices
- **Microlending Contract**: Can be integrated with the microlending contract for emergency loans when payouts are insufficient
- **Cooperative Management**: Works with the cooperative management contract for governance decisions

## 📚 References

- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.