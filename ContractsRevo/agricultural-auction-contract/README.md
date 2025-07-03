# Agricultural Auction Contract

## 🎯 Overview
The Agricultural Auction Contract is a decentralized smart contract built on the Soroban framework for the Stellar blockchain. It enables farmers to list their agricultural products for auction, allowing buyers to place bids and purchase products. The contract ensures transparency, fair pricing, and quality verification in agricultural product sales by leveraging blockchain technology.

## 📜 Features
- Farmers can list agricultural products with detailed information
- Dynamic pricing based on product freshness and quality
- Auction system with bidding functionality
- Seasonal status verification for products
- Quality grading and freshness rating systems
- Storage condition monitoring
- Market price oracle integration
- Bulk purchase discounts

## 🛠 Contract Functionality
### **1. Product Management**
The `ProductListing` module allows farmers to:
- Add new agricultural products with detailed information
- Update product freshness ratings as time passes
- Update product quantities as inventory changes
- Update quality grades based on inspections
- Calculate expiry dates based on product type
- Verify seasonal status of products by region

### **2. Auction System**
The `AuctionOperations` module enables:
- Creating auctions for listed products
- Placing bids on auctioned products
- Extending auction end times
- Finalizing auctions and transferring ownership
- Bulk purchase discounts for larger quantity bids
- Dynamic pricing based on market conditions

### **3. Price Oracle**
The contract includes a price oracle system that:
- Provides current market prices for different product types by region
- Tracks price trends over time
- Helps farmers set competitive reserve prices
- Enables buyers to evaluate bid fairness

### **4. Quality Verification**
The contract includes quality verification features:
- Products are assigned quality grades (Premium to Rejected)
- Freshness ratings based on harvest date
- Storage condition monitoring
- Certification verification

### **5. Time Management**
The contract handles time-sensitive operations:
- Calculating product expiry dates based on product type
- Ensuring auctions end before products expire
- Tracking harvest dates and product age
- Seasonal status verification

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
   cd ContractsRevo/agricultural-auction-contract
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
### **AgriculturalProduct**
Represents a farm product with the following properties:
- ID, farmer address, name, description
- Base price and current price (adjusted by quality/freshness)
- Weight, quantity, harvest date, expiry date
- Images, freshness rating, quality grade
- Certifications, storage condition
- Product type, region, seasonal status

### **Auction**
Represents an auction for a product:
- Product ID, farmer address
- Highest bid and bidder
- Reserve price, auction end time
- Quantity available, minimum quantity
- Bulk discount threshold and percentage
- Dynamic pricing flag

### **MarketPrice**
Tracks market prices for products:
- Product type, region
- Current price, timestamp
- Price trend, trading volume

## 📌 Best Practices
- Ensure product descriptions are accurate and detailed
- Set reasonable reserve prices based on market conditions
- Monitor product freshness and update ratings regularly
- Verify seasonal status before listing products
- Set appropriate auction end times before product expiry
- Consider bulk discounts for larger quantity sales

## 📖 Error Handling
The contract includes comprehensive error handling for:
- Admin operations (unauthorized access)
- Auction operations (invalid bids, expired products)
- Product management (invalid descriptions, out of season)
- Quality verification (invalid grades, certification issues)
- Price oracle (unavailable price data)
- Time management (future harvest dates, expired products)

## 🔄 Contract Interactions
### **For Farmers**
1. Add products with detailed information
2. Create auctions with reserve prices
3. Update product freshness and quality
4. Extend auctions if needed
5. Finalize auctions after completion

### **For Buyers**
1. Browse available products
2. Check market prices and product quality
3. Place bids on auctions
4. Purchase products in bulk with discounts
5. Verify product certifications and freshness

## 📈 Market Integration
The contract integrates with market data to:
- Provide current market prices for products
- Track price trends by region
- Adjust prices based on seasonal status
- Enable dynamic pricing for auctions

## 🌐 Use Cases
- Direct farmer-to-consumer sales
- Wholesale agricultural marketplaces
- Cooperative selling platforms
- Farm-to-table restaurant supply chains
- Organic and specialty product marketplaces

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
