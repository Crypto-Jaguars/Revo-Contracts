# CSA Membership Contract

## 🎯 Overview
The CSA (Community Supported Agriculture) Membership Contract is a smart contract built on the Soroban framework for the Stellar blockchain. It enables farms to manage membership subscriptions for their CSA programs, allowing consumers to directly support local agriculture through seasonal share purchases. The contract facilitates transparent management of CSA memberships, including enrollment, updates, and cancellations.

## 📜 Features
- Membership enrollment with farm-specific details
- Multiple share size options (Small, Medium, Large)
- Seasonal membership management
- Pickup location assignment and updates
- Membership verification and validation
- Cancellation handling with proper authorization
- Comprehensive metadata storage and retrieval

## 🛠 Contract Functionality
### **1. Membership Enrollment**
The contract allows farms and members to:
- Create new CSA memberships for specific farms
- Specify the farming season for the membership
- Choose share sizes based on member preferences
- Set pickup locations for produce distribution
- Define membership duration with start and end dates
- Validate season and date parameters
- Generate unique membership tokens

### **2. Membership Management**
Members can manage their CSA subscriptions by:
- Updating pickup locations as needed
- Retrieving membership details and status
- Verifying membership validity
- Tracking membership across multiple seasons
- Managing membership parameters

### **3. Membership Cancellation**
The contract provides functionality to:
- Cancel memberships with proper authorization
- Verify member identity before cancellation
- Remove cancelled memberships from storage
- Emit events for membership cancellation
- Handle error cases appropriately

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
   cd ContractsRevo/csa-membership-contract
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
### **CSAMembership**
Represents a CSA membership with the following properties:
- Farm ID: Unique identifier for the farm
- Season: The growing season for the membership (e.g., "Summer 2025")
- Share Size: Size of the CSA share (Small, Medium, Large)
- Pickup Location: Where members collect their produce
- Start Date: Beginning of the membership period
- End Date: End of the membership period
- Member: Address of the CSA member

### **ShareSize**
Enum representing the available share sizes:
- Small: For individuals or small households
- Medium: For average-sized households
- Large: For large households or shared memberships

## 📌 Best Practices
- Ensure proper authorization before modifying membership details
- Validate date ranges for seasonal memberships
- Verify farm existence and validity before enrollment
- Use unique identifiers for memberships
- Implement proper error handling for all operations
- Emit events for important membership actions
- Maintain accurate metadata for all memberships

## 📖 Error Handling
The contract includes comprehensive error handling for:
- Membership operations (not found, already cancelled)
- Authorization (unauthorized access to memberships)
- Validation (invalid dates, farms, or seasons)
- Data integrity (ensuring proper membership structure)

## 🔄 Contract Interactions
### **For Farms**
1. Set up farm profiles and seasonal offerings
2. Manage available share sizes and pickup locations
3. Track member enrollments and cancellations
4. Verify membership validity

### **For Members**
1. Enroll in CSA programs for specific farms and seasons
2. Update pickup locations as needed
3. View membership details and status
4. Cancel memberships when necessary

## 🌐 Use Cases
- Seasonal CSA program management for small farms
- Multi-farm CSA networks with shared distribution
- Year-round CSA programs with seasonal variations
- Specialty CSA programs (e.g., vegetable, fruit, flower, meat)
- Farm-to-table restaurant subscription programs
- Educational farm programs with membership components
- Urban agriculture CSA initiatives

## 📚 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
