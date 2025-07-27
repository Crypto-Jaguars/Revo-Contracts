# Water Management Contract

## 🎯 Overview
The Water Management Contract is a smart contract built on the Soroban framework for the Stellar blockchain. It tracks and incentivizes efficient water usage in agriculture, ensuring sustainable practices and regulatory compliance. The contract integrates IoT sensor data, provides rewards for efficient usage, and generates alerts for excessive consumption.

## 📜 Features
- **Water Usage Tracking**: Record water consumption per parcel using IoT sensor data
- **Incentive System**: Reward farmers with loyalty tokens for efficient water usage
- **Alert Generation**: Issue alerts for excessive water consumption
- **Threshold Management**: Set and manage acceptable water usage limits
- **Comprehensive Reporting**: Generate detailed usage reports for farmers and parcels
- **Oracle Integration**: Support for off-chain sensor data with on-chain verification
- **Environmental Integration**: Compatible with environmental-impact-tracking contract
- **Loyalty Token Integration**: Seamless integration with loyalty-token-contract

## 🛠 Contract Functionality

### **1. Water Usage Tracking**
The contract allows farmers to:
- Record water consumption data with IoT sensor integration
- Store consumption summaries on-chain with detailed data hashes
- Track usage per parcel or crop with timestamp verification
- Maintain accurate records for regulatory reporting
- Associate usage with specific agricultural parcels

### **2. Incentive System**
Farmers can earn rewards through:
- Automatic incentive processing for efficient usage
- Loyalty token rewards based on efficiency scores
- Threshold-based reward calculations
- Integration with existing loyalty programs
- Transparent reward distribution

### **3. Alert System**
The contract provides:
- Automatic alert generation for excessive consumption
- Threshold-based monitoring (daily, weekly, monthly)
- Real-time notifications for farmers
- Alert resolution tracking
- Multiple alert types for different scenarios

### **4. Threshold Management**
Administrators can:
- Set water usage limits per parcel
- Configure daily, weekly, and monthly thresholds
- Update limits based on seasonal requirements
- Monitor compliance across multiple farms
- Ensure regulatory adherence

## 🚀 Setup Guide

### **Prerequisites**
Ensure you have the following installed:
- Rust & Cargo
- Soroban CLI
- Stellar Standalone/Testnet/Mainnet access

### **Installation Steps**
1. **Clone the Repository**
   ```bash
   git clone https://github.com/Crypto-Jaguars/Revo-Contracts.git
   cd ContractsRevo/water-management-contract
   ```

2. **Build the Contract**
   ```bash
   stellar contract build
   ```

3. **Run the Tests**
   ```bash
   cargo test
   ```

4. **Deploy the Contract**
   ```bash
   stellar contract deploy --wasm target/wasm32-unknown-unknown/release/water_management_contract.wasm
   ```

## 📊 Data Structures

### **WaterUsage**
```rust
pub struct WaterUsage {
    pub usage_id: BytesN<32>,
    pub farmer_id: Address,
    pub parcel_id: BytesN<32>,
    pub volume: i128, // Water volume in liters
    pub timestamp: u64,
    pub data_hash: BytesN<32>, // Hash of off-chain sensor data
}
```

### **Incentive**
```rust
pub struct Incentive {
    pub farmer_id: Address,
    pub reward_amount: i128,
    pub timestamp: u64,
    pub usage_id: BytesN<32>,
}
```

### **WaterThreshold**
```rust
pub struct WaterThreshold {
    pub parcel_id: BytesN<32>,
    pub daily_limit: i128,
    pub weekly_limit: i128,
    pub monthly_limit: i128,
}
```

### **Alert**
```rust
pub struct Alert {
    pub alert_id: BytesN<32>,
    pub farmer_id: Address,
    pub parcel_id: BytesN<32>,
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: u64,
    pub resolved: bool,
}
```

## 🔑 Key Functions

### **Core Functions**
- `record_usage()` – Record water usage data for a parcel or crop
- `issue_incentive()` – Reward farmers for efficient water usage
- `generate_alert()` – Issue alerts for excessive water consumption
- `get_usage_report()` – Retrieve water usage reports for a farmer or parcel
- `set_threshold()` – Update acceptable water usage thresholds

### **Query Functions**
- `get_usage()` – Get specific water usage record
- `get_farmer_usages()` – Get all usage records for a farmer
- `get_parcel_usages()` – Get all usage records for a parcel
- `get_threshold()` – Get water usage threshold for a parcel
- `get_incentive()` – Get incentive record by usage ID
- `get_alert()` – Get alert by ID

### **Management Functions**
- `initialize()` – Initialize contract with admin
- `resolve_alert()` – Mark alert as resolved
- `calculate_farmer_rewards()` – Calculate total rewards for a period

## 🔄 Contract Interactions

### **For Farmers**
1. Record water usage with IoT sensor data
2. Monitor efficiency scores and thresholds
3. Earn loyalty tokens for efficient usage
4. Receive alerts for excessive consumption
5. Track usage history and rewards

### **For Administrators**
1. Set water usage thresholds per parcel
2. Monitor overall water consumption
3. Generate compliance reports
4. Manage alert systems
5. Configure incentive parameters

### **For IoT Systems**
1. Submit sensor data with cryptographic hashes
2. Verify data integrity on-chain
3. Trigger automatic processing
4. Monitor sensor health and accuracy

## 🌐 Integration

### **Environmental Impact Tracking**
- Automatic carbon credit calculations
- Water conservation impact reporting
- Sustainability metrics tracking
- Environmental compliance monitoring

### **Loyalty Token Contract**
- Seamless reward distribution
- Point accumulation for efficient usage
- Redemption options for farmers
- Loyalty program integration

### **Oracle Integration**
- Chainlink oracle support for IoT data
- Off-chain data verification
- Real-time sensor monitoring
- Data integrity assurance

## 📈 Use Cases

- **Precision Agriculture**: Optimize water usage with real-time monitoring
- **Regulatory Compliance**: Meet water usage regulations and reporting requirements
- **Sustainability Programs**: Incentivize water conservation practices
- **Insurance Integration**: Provide usage data for agricultural insurance
- **Supply Chain Transparency**: Track water usage in agricultural supply chains
- **Research and Analytics**: Generate insights on water usage patterns

## 🔒 Security Features

- **Authentication**: Farmer and admin authorization required
- **Data Integrity**: Cryptographic hashes for off-chain data
- **Access Control**: Role-based permissions
- **Input Validation**: Comprehensive parameter validation
- **Error Handling**: Robust error management

## 📚 References

- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Chainlink Oracles](https://docs.chain.link/docs/stellar/)

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines and submit pull requests for any improvements.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
