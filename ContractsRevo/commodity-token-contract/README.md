# Commodity Token Contract

## 🎯 Overview
The Commodity Token Contract is a smart contract built on the Soroban framework for the Stellar blockchain. It enables the tokenization of physical agricultural commodities, creating digital representations that can be traded, transferred, and redeemed. This contract bridges the gap between physical agricultural assets and digital markets, enhancing liquidity and transparency in agricultural commodity trading.

## 📜 Features
- Issue digital tokens backed by physical commodities
- Track commodity inventory and token ownership
- Validate commodity authenticity through verification data
- Manage token lifecycle including expiration and redemption
- Support for different commodity types and grades
- Detailed metadata for each tokenized commodity
- Authorized issuer management for regulatory compliance

## 🛠 Contract Functionality
### **1. Token Issuance**
The contract allows authorized issuers to:
- Create digital tokens representing physical commodities
- Specify commodity type, quantity, grade, and storage location
- Set expiration dates for perishable commodities
- Include verification data for authenticity validation
- Generate unique token IDs using secure hashing algorithms

### **2. Token Redemption**
Users can redeem tokens to claim the underlying physical commodities:
- Verify token ownership before redemption
- Redeem full or partial quantities of tokens
- Update inventory records automatically upon redemption
- Handle token expiration checks during redemption
- Emit events for redemption tracking

### **3. Inventory Management**
The contract provides functionality to:
- Track total and available inventory for each commodity type
- Update inventory levels when tokens are issued or redeemed
- Prevent token issuance when inventory is insufficient
- Add new inventory by authorized administrators
- List available inventory by commodity type

### **4. Commodity Verification**
The contract includes verification capabilities:
- Register verification data for commodity types
- Validate commodity authenticity during token issuance
- Store metadata about verification standards
- Support for multiple verification methods
- Secure hash-based verification

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
   cd ContractsRevo/commodity-token-contract
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
### **CommodityBackedToken**
Represents a tokenized commodity with the following properties:
- Commodity Type: String identifier for the commodity (e.g., "Coffee", "Cocoa")
- Quantity: Amount of the commodity represented by the token
- Grade: Quality grade of the commodity
- Storage Location: Where the physical commodity is stored
- Expiration Date: When the token or underlying commodity expires
- Verification Data: Cryptographic hash for authenticity verification

### **Inventory**
Tracks the inventory status for a commodity type:
- Total Quantity: Total amount of the commodity in the system
- Available Quantity: Amount available for tokenization
- Issued Tokens: Amount already tokenized

## 📌 Best Practices
- Ensure proper authorization before issuing or redeeming tokens
- Regularly audit inventory levels against physical commodities
- Set appropriate expiration dates for perishable commodities
- Use secure methods to generate verification data
- Implement proper access controls for administrative functions
- Maintain accurate metadata for all tokenized commodities

## 📖 Error Handling
The contract includes comprehensive error handling for:
- Token issuance (unauthorized issuer, invalid data, insufficient inventory)
- Token redemption (token not found, insufficient quantity, expired tokens)
- Inventory management (underflow, overflow)
- Authorization (unauthorized access to admin functions)
- Token ID generation (nonce overflow, generation errors)

## 🔄 Contract Interactions
### **For Administrators**
1. Initialize the contract
2. Add authorized issuers
3. Add inventory for commodity types
4. Register commodity verification data

### **For Issuers**
1. Issue tokens for commodities
2. Validate commodity authenticity
3. Manage token metadata

### **For Token Holders**
1. View token details and metadata
2. Redeem tokens for physical commodities
3. Transfer tokens to other users (via external token standards)

## 🌐 Use Cases
- Tokenizing agricultural commodities for digital trading
- Creating liquid markets for physical commodities
- Enabling fractional ownership of commodity batches
- Streamlining supply chain tracking and verification
- Supporting commodity-backed financial instruments
- Facilitating transparent pricing and market access

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
