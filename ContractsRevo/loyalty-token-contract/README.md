# Loyalty Token Contract

## 🎯 Overview
The Loyalty Token Contract is a smart contract built on the Soroban framework for the Stellar blockchain. It enables agricultural businesses to create and manage customer loyalty programs, allowing them to reward customers for their purchases and engagement. The contract facilitates point accumulation, reward redemption, and program management in a transparent and efficient manner.

## 📜 Features
- Customizable loyalty program creation
- Configurable points-per-transaction ratio
- Multiple redemption options with varying point requirements
- Automatic point calculation and award
- Secure reward redemption process
- Inventory management for available rewards
- Transparent point balance tracking
- Event emission for key actions

## 🛠 Contract Functionality
### **1. Program Management**
The contract allows businesses to:
- Create new loyalty programs with unique identifiers
- Define points awarded per transaction amount
- Configure multiple redemption options with different values
- Set available quantities for each reward
- Retrieve program information and configuration
- List available rewards and their requirements

### **2. Point Earning**
Users can earn points through:
- Purchase transactions that award points automatically
- Point calculation based on transaction amount
- Secure point storage linked to user addresses
- Transparent point balance tracking
- Event emission for point awards

### **3. Reward Redemption**
The contract provides functionality to:
- Redeem points for available rewards
- Verify point balance sufficiency
- Check reward availability
- Automatically update point balances after redemption
- Decrement available reward quantities
- Emit events for redemption tracking

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
   cd ContractsRevo/loyalty-token-contract
   ```
2. **Build the Contract**
   ```bash
   stellar contract build
   ```
3. **Run the Tests**
   ```bash
   cargo test
   ```

## 📊 Data Structures
### **LoyaltyProgram**
```rust
pub struct LoyaltyProgram {
    pub program_id: BytesN<32>,
    pub points_per_transaction: u32,
    pub redemption_options: Vec<RedemptionOption>,
}
```

### **RedemptionOption**
```rust
pub struct RedemptionOption {
    pub id: u32,
    pub name: String,
    pub points_required: u32,
    pub available_quantity: u32,
}
```

## 📌 Best Practices
- Create unique program IDs for different loyalty initiatives
- Set appropriate points-per-transaction ratios based on business economics
- Offer a variety of redemption options at different point levels
- Regularly update reward inventories
- Monitor point accumulation patterns
- Implement proper authorization for program management
- Emit and monitor events for program activity

## 📖 Error Handling
The contract includes comprehensive error handling for:
- Program operations (program not found)
- Redemption operations (option not found, out of stock)
- Point operations (insufficient points)
- Authorization (unauthorized access to program management)
- Data integrity (ensuring proper program structure)

## 🔄 Contract Interactions
### **For Businesses**
1. Create a loyalty program with appropriate configuration
2. Define attractive redemption options
3. Award points for customer transactions
4. Monitor point accumulation and redemption patterns
5. Replenish reward inventories as needed

### **For Customers**
1. Earn points through purchases and engagement
2. View available rewards and their point requirements
3. Check point balances
4. Redeem points for desired rewards
5. Track redemption history

## 🌐 Use Cases
- Farm-direct purchase reward programs
- CSA membership loyalty benefits
- Farmers market regular customer incentives
- Agricultural cooperative member rewards
- Seasonal purchase incentives
- Product-specific loyalty programs
- Cross-farm loyalty networks
- Sustainable farming practice incentives

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
