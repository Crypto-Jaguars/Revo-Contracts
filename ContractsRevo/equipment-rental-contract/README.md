# Equipment Rental Contract

## 🎯 Overview
The Equipment Rental Contract is a smart contract built on the Soroban framework for the Stellar blockchain. It enables agricultural equipment owners to list their machinery for rental, while allowing farmers to rent equipment for specific time periods. The contract manages the entire rental lifecycle, from equipment registration and maintenance tracking to rental agreements and pricing calculations.

- contractId: CACWW3GDQVFQJTVLGHCMZMWI6QT5XFYU2NLOYQ3KQKAHPV2D4T4BPBQG
- Link: https://stellar.expert/explorer/testnet/contract/CACWW3GDQVFQJTVLGHCMZMWI6QT5XFYU2NLOYQ3KQKAHPV2D4T4BPBQG

## 📜 Features
- Equipment registration with detailed metadata
- Real-time availability management
- Maintenance status tracking and history
- Complete rental lifecycle management
- Dynamic pricing calculations
- Comprehensive rental history tracking
- User-specific rental records
- Transparent maintenance logging

## 🛠 Contract Functionality
### **1. Equipment Management**
The contract allows equipment owners to:
- Register new equipment with unique identifiers
- Specify equipment types and daily rental prices
- Set and update equipment locations
- Mark equipment as available or unavailable
- Track maintenance status (Good, NeedsService, UnderMaintenance)
- Log detailed maintenance records with timestamps and notes
- Retrieve comprehensive maintenance history

### **2. Rental Lifecycle**
Users can manage the complete rental process:
- Create rental requests for specific date ranges
- Confirm and activate rentals
- Complete rentals and release equipment
- Cancel rentals before the start date
- Track rental status throughout the lifecycle
- View rental history by equipment or user

### **3. Pricing and Validation**
The contract provides sophisticated pricing functionality:
- Compute total rental prices based on duration
- Validate proposed prices against expected calculations
- Apply pricing tolerance for negotiation flexibility
- Ensure transparent and fair pricing for all parties

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
   cd ContractsRevo/equipment-rental-contract
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
### **Equipment**
```rust
struct Equipment {
    id: BytesN<32>,
    equipment_type: String,
    owner: Address,
    rental_price_per_day: i128,
    available: bool,
    location: String,
    maintenance_status: MaintenanceStatus,
}
```

### **Rental**
```rust
struct Rental {
    equipment_id: BytesN<32>,
    renter: Address,
    start_date: u64,
    end_date: u64,
    total_price: i128,
    status: RentalStatus,
}
```

### **MaintenanceStatus**
```rust
enum MaintenanceStatus {
    Good,
    NeedsService,
    UnderMaintenance,
}
```

### **RentalStatus**
```rust
enum RentalStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}
```
## 📌 Best Practices
- Ensure proper authentication before modifying equipment or rental details
- Validate date ranges for rental periods
- Block rentals for equipment under maintenance
- Implement proper error handling for all operations
- Maintain accurate maintenance records
- Verify price calculations before confirming rentals
- Use unique identifiers for equipment and rentals

## 📖 Error Handling
The contract includes comprehensive error handling for:
- Equipment operations (not found, already registered)
- Rental operations (invalid dates, unavailable equipment)
- Authorization (unauthorized access to equipment)
- Pricing (invalid calculations, price mismatches)
- Maintenance (invalid status updates)

## 🔄 Contract Interactions
### **For Equipment Owners**
1. Register equipment with detailed specifications
2. Update equipment availability as needed
3. Track and log maintenance events
4. Confirm and complete rental agreements
5. View rental and maintenance history

### **For Renters**
1. Browse available equipment
2. Create rental requests for specific periods
3. Calculate expected rental costs
4. Cancel rentals when necessary
5. View personal rental history

## 🌐 Use Cases
- Seasonal equipment sharing among farmers
- Cooperative equipment pools
- Specialized agricultural machinery rental
- Farm-to-farm equipment lending
- Community tool libraries for small-scale farmers
- Equipment testing before purchase
- Maintenance tracking for shared equipment

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
