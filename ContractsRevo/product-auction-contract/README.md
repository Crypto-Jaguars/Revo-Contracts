# Product Auction Contract

## 🎯 Overview
The Product Auction Contract is a smart contract built on the Soroban framework for the Stellar blockchain. It enables agricultural producers to list and auction their products, while providing buyers with a transparent and secure marketplace. The contract facilitates the entire auction lifecycle, from product listing and bidding to shipment tracking and dispute resolution.

## 📜 Features
- Product listing with detailed metadata
- Auction creation and management
- Secure bidding mechanism
- Shipment tracking and cost calculation
- Seller verification system
- Product condition verification
- Dispute resolution process
- Return policy management
- Comprehensive event tracking

## 🛠 Contract Functionality
### **1. Product Management**
The contract allows sellers to:
- Add products with detailed descriptions
- Specify product conditions (New, OpenBox, UsedGood, etc.)
- Set pricing and stock levels
- Upload product images
- Update inventory levels
- Set return policies
- Track product verification status

### **2. Auction Operations**
Users can participate in auctions through:
- Creating auctions with reserve prices
- Setting auction end times
- Placing bids on products
- Extending auction durations
- Finalizing auctions with winner determination
- Automatic inventory updates after successful auctions

### **3. Shipping and Logistics**
The contract provides functionality to:
- Calculate shipping costs based on weight and distance
- Estimate delivery times
- Create shipment records with tracking numbers
- Update shipping status throughout delivery
- Verify buyer locations and shipping zones
- Track shipment history

### **4. Verification and Dispute Resolution**
The system includes robust verification mechanisms:
- Seller verification process
- Product authenticity verification
- Condition verification by administrators
- Dispute filing and resolution
- Return request management
- Transparent resolution tracking

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
   cd ContractsRevo/product-auction-contract
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
### **Product**
```rust
pub struct Product {
    pub id: u64,
    pub seller: Address,
    pub name: Symbol,
    pub description: String,
    pub price: u64,
    pub condition: Condition,
    pub stock: u32,
    pub images: Vec<String>,
    pub weight_pounds: u64,
    pub verified: bool,
}
```

### **Auction**
```rust
pub struct Auction {
    pub product_id: u64,
    pub highest_bid: u64,
    pub highest_bidder: Option<Address>,
    pub reserve_price: u64,
    pub auction_end_time: u64,
    pub seller: Address,
}
```

### **Shipment**
```rust
pub struct Shipment {
    pub seller: Address,
    pub buyer: Address,
    pub weight_pounds: u32,
    pub distance_km: u32,
    pub shipping_cost: u64,
    pub delivery_estimate_days: u32,
    pub status: Symbol,
    pub tracking_number: String,
}
```

### **Dispute**
```rust
pub struct Dispute {
    pub buyer: Address,
    pub seller: Address,
    pub product_id: u64,
    pub reason: String,
    pub status: DisputeStatus,
}
```

## 📌 Best Practices
- Ensure proper authentication before modifying product or auction details
- Validate all parameters before creating auctions
- Set appropriate reserve prices based on market conditions
- Provide accurate product descriptions and conditions
- Include clear return policies
- Respond promptly to verification requests
- Track shipping status updates
- Handle disputes professionally and transparently

## 📖 Error Handling
The contract includes comprehensive error handling for:
- Auction operations (bid too low, auction ended, etc.)
- Product operations (invalid descriptions, out of stock)
- Shipping operations (restricted locations, invalid zones)
- Verification operations (disputes, return requests)
- Administrative operations (unauthorized access)

## 🔄 Contract Interactions
### **For Sellers**
1. List products with detailed descriptions and images
2. Create auctions with appropriate reserve prices
3. Track bids and finalize auctions
4. Manage shipments and update shipping status
5. Handle returns and disputes
6. Maintain verification status

### **For Buyers**
1. Browse available products and auctions
2. Place bids on desired items
3. Track shipments of purchased products
4. File disputes if necessary
5. Request returns according to policies

### **For Administrators**
1. Verify sellers and products
2. Resolve disputes between buyers and sellers
3. Monitor marketplace activity
4. Ensure compliance with platform rules

## 🌐 Use Cases
- Direct farm-to-consumer produce auctions
- Specialty agricultural product marketplaces
- Seasonal harvest auctions
- Cooperative selling platforms
- Artisanal food product marketplaces
- Agricultural equipment auctions
- Rare or heritage crop auctions
- Community-supported agriculture distribution

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
