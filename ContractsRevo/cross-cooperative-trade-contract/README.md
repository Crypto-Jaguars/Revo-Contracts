# Cross-Cooperative Trade Contract

## 🎯 Overview
The Cross-Cooperative Trade Contract is a decentralized smart contract built on the Soroban framework for the Stellar blockchain. It facilitates trade and barter exchanges between agricultural cooperatives, enabling them to trade agricultural products, resources, and services in a transparent and trustless manner. The contract manages trade offers, barter agreements, and reputation systems to ensure fair and reliable inter-cooperative commerce.

## 📜 Features
- Cooperatives can create trade offers for agricultural products
- Barter system for direct product exchanges without monetary transactions
- Reputation tracking system for cooperatives based on trade history
- Secure trade completion with multi-party verification
- Active offer management and discovery system
- Result-based error handling for robust operation
- Comprehensive audit trail for all trade activities

## 🛠 Contract Functionality
### **1. Trade Management**
The `Trade` module enables cooperatives to:
- Create trade offers specifying offered and requested products
- Accept trade offers from other cooperatives
- Complete trades with proper authorization verification
- List all active trade offers for discovery
- Get detailed information about specific trade offers
- Track trade status through the entire lifecycle

### **2. Barter Agreement System**
The `Barter` module manages:
- Creation of formal barter agreements between cooperatives
- Agreement status tracking (Active, Completed, Disputed)
- Linking trade offers to barter agreements
- Multi-party agreement verification
- Agreement lifecycle management

### **3. Reputation System**
The contract includes a comprehensive reputation system that:
- Tracks successful trades for each cooperative
- Maintains rating scores on a 1-5 scale
- Updates reputation automatically after each trade
- Provides trust scores for cooperative assessment
- Enables reputation-based trade partner selection
- Supports external rating system integration

### **4. Administrative Functions**
The contract provides administrative capabilities:
- Contract initialization with admin setup
- Admin authorization verification
- Configuration management
- System maintenance operations

### **5. Error Handling**
Robust error handling system with:
- Admin-specific errors (AlreadyInitialized, UnauthorizedAccess, NotInitialized)
- Trade-specific errors (TradeOfferNotFound, InvalidTradeStatus, TradeExpired, etc.)
- Result-based return values for all operations
- Comprehensive error reporting and debugging

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
   cd ContractsRevo/cross-cooperative-trade-contract
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
### **TradeOffer**
Represents a trade offer between cooperatives:
- `offer_id`: Unique identifier for the trade offer
- `cooperative_id`: Address of the cooperative making the offer
- `offered_product`: Hash identifier of the product being offered
- `requested_product`: Hash identifier of the product being requested
- `status`: Current status ("Pending", "Accepted", "Completed")

### **BarterAgreement**
Represents a formal barter agreement:
- `agreement_id`: Unique identifier for the agreement
- `trade_offer_id`: Reference to the associated trade offer
- `offering_cooperative`: Address of the cooperative making the offer
- `accepting_cooperative`: Address of the cooperative accepting the offer
- `status`: Agreement status ("Active", "Completed", "Disputed")

### **Reputation**
Tracks cooperative reputation and trustworthiness:
- `cooperative_id`: Address of the cooperative
- `successful_trades`: Number of successfully completed trades
- `rating`: Reputation rating on a 1-5 scale based on trade history

## 📌 Best Practices
- Ensure proper authorization before creating trade offers
- Verify product availability before making trade offers
- Update reputation after each completed trade
- Monitor active offers and respond promptly to accepted trades
- Use descriptive product identifiers for clear trade terms
- Maintain good standing through successful trade completion

## 📖 Error Handling
The contract includes comprehensive error handling for:
- **Admin Operations**: Unauthorized access, double initialization, uninitialized contract
- **Trade Operations**: Invalid trade offers, expired trades, unauthorized access
- **Reputation Management**: Invalid cooperative addresses, calculation errors
- **Barter Agreements**: Missing agreements, invalid status transitions

## 🔄 Contract Interactions
### **For Cooperative Administrators**
1. Initialize the contract with admin privileges
2. Monitor system-wide trade activities
3. Manage contract configuration and updates
4. Handle dispute resolution when necessary

### **For Cooperatives**
1. Create trade offers for available products
2. Browse and accept trade offers from other cooperatives
3. Complete trades by fulfilling agreement terms
4. Build reputation through successful trades
5. Monitor reputation scores and trust metrics

## 📈 Trade Lifecycle
1. **Offer Creation**: Cooperative creates a trade offer specifying products
2. **Offer Discovery**: Other cooperatives browse active offers
3. **Offer Acceptance**: Interested cooperative accepts the trade offer
4. **Agreement Formation**: Barter agreement is automatically created
5. **Trade Completion**: Both parties fulfill their obligations
6. **Reputation Update**: System updates reputation scores for both parties

## 🌐 Use Cases
- Direct cooperative-to-cooperative agricultural product exchanges
- Resource sharing between farming cooperatives
- Equipment and tool lending/borrowing arrangements
- Seasonal crop exchanges based on harvest timing
- Specialty product trading networks
- Regional cooperative marketplaces
- Bulk commodity exchanges
- Emergency resource sharing during crises

## 🔒 Security Features
- Multi-signature authorization requirements
- Cryptographic offer and agreement IDs
- Immutable trade history and reputation records
- Result-based error handling preventing contract failures
- Comprehensive access control mechanisms
- Transparent and auditable trade processes

## 📚 API Reference
### **Core Functions**
- `initialize(admin: Address)` - Initialize contract with admin
- `create_trade_offer(cooperative_id, offered_product, requested_product)` - Create new trade offer
- `accept_trade(offer_id, accepting_cooperative)` - Accept existing trade offer
- `complete_trade(offer_id, caller)` - Complete a trade transaction
- `get_trade_details(offer_id)` - Retrieve trade offer information
- `list_active_offers()` - Get all active trade offers
- `get_barter_agreement(agreement_id)` - Retrieve barter agreement details
- `update_reputation(cooperative_id, successful)` - Update cooperative reputation

### **Reputation Functions**
- `get_reputation(cooperative_id)` - Get cooperative reputation details
- `calculate_trust_score(cooperative_id)` - Calculate trust score
- `is_cooperative_trustworthy(cooperative_id)` - Check trustworthiness
- `get_reputation_summary(cooperative_id)` - Get comprehensive reputation data

## 🧪 Testing
The contract includes comprehensive test coverage:
- 17+ test cases covering all major functionality
- Error condition testing for robust operation
- Integration tests for multi-party scenarios
- Reputation system validation tests
- Trade lifecycle end-to-end tests

Run tests with:
```bash
cargo test
```

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Agricultural Cooperative Trade Networks](https://www.ica.coop/)

