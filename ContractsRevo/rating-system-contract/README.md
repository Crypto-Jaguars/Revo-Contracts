# Rating System Contract

## 🎯 Overview
The Rating System Contract is a decentralized smart contract built on the Soroban framework for the Stellar blockchain. It allows buyers to rate sellers based on their transaction experience. The contract ensures transparency, reliability, and integrity in rating data by leveraging blockchain technology.

## 📜 Features
- Buyers can submit ratings for sellers.
- Ratings include a score (1-5), weight, and optional feedback.
- Sellers' ratings are stored and updated transparently on the blockchain.
- Weighted ratings are calculated for fair assessments.
- Reputation scores are derived from weighted ratings.
- Events are triggered for tracking rating submissions and updates.

## 🛠 Contract Functionality
### **1. Submitting a Rating**
Buyers can rate sellers using the `rate_seller_system` function, which:
- Validates the rating value (must be between 1 and 5).
- Creates a `Rating` record that includes buyer, rating, weight, and optional feedback.
- Stores the rating on-chain and triggers an event.

### **2. Updating Weighted Ratings**
The `update_weighted_rating` function:
- Fetches the existing weighted rating and total weight for a seller.
- Updates the total weighted rating by incorporating the new rating and weight.
- Saves the updated values and triggers an event.

### **3. Calculating Weighted Ratings**
The `calculate_weighted_rating` function:
- Retrieves the total weighted rating and weight of a seller.
- Computes the weighted rating as `total_weighted_rating / total_weight`.
- Triggers an event for tracking calculated ratings.

### **4. Calculating Reputation Score**
The `reputation_score_calculate` function:
- Fetches the weighted rating for the seller.
- Determines the reputation score based on the rating range.
- Returns a reputation score between 1 and 5.

### **5. Adding Reputation History**
The `add_reputation_score_history` function:
- Retrieves the seller's existing reputation history or initializes a new one.
- Creates a new `ReputationRecord` with the score and timestamp.
- Stores the record and triggers an event.

### **6. Retrieving Rating History**
The `get_rating_history` function:
- Fetches all past ratings for a given seller.
- Returns the stored rating records.

### **7. Retrieving Reputation History**
The `get_reputation_history` function:
- Fetches the historical reputation records for a seller.
- Returns the stored reputation records.

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
   cd Revo-Contracts/ContractsRevo/rating-system-contract/src
   ```
2. **Build the Contract**
   ```bash
 stellar contract build
   ```
3. **Run the Tests**
```bash
cargo test
 ```
### Usage
This contract provides a **decentralized reputation system** for a marketplace on **Soroban**. It enables buyers to rate sellers, store these ratings, compute **weighted ratings**, and track **reputation scores** over time.  

It is useful for:  
✅ **Marketplaces:** Buyers can rate sellers, ensuring trust and reliability.  
✅ **Lending Platforms:** Users can build reputation scores for credibility.  
✅ **Freelance/Gig Economy:** Clients can rate service providers.  
✅ **DeFi Protocols:** Trust-based systems can use this for assessing risk.  

---

### **How It Works**  

**1. Buyers Rate Sellers**  
- Buyers submit a **rating (1–5)**, **weight**, and optional **feedback**.  
- These ratings are stored in **Soroban’s contract storage**.  
- Events are emitted for tracking.  

**2. Weighted Rating Calculation**  
- A seller’s overall rating is computed using **weighted averages**.  
- Higher-weight ratings contribute more significantly.  

**3. Reputation Score Calculation**  
- A seller’s weighted rating is mapped to a **1–5 reputation score**.  
- This score reflects the seller’s overall trustworthiness.  

**4. Reputation History Tracking**  
- Reputation scores are **time-stamped** and stored in history.  
- This allows buyers to see how a seller's reputation evolves over time.  

---

## **Who Can Use It?**  

### 🏪 **Marketplaces** 
- Sellers build a **trust score** to attract buyers.  
- Buyers can **check ratings before purchasing**.  

### 💰 **Lending & Credit Platforms**  
- Lenders can use **reputation scores** to decide loan eligibility.  

### 🎭 **Freelance & Gig Economy**  
- Service providers gain **credibility** based on client ratings.  

### 🌉 **DeFi & DAO Governance**  
- Users with high **reputation scores** can access premium services.  

---

This contract ensures **trust, transparency, and accountability** in decentralized systems. 🚀 Let me know if you need any refinements! 😊
## 📌 Best Practices
- Ensure ratings are within the allowed range (1-5).
- Use unique seller identifiers to avoid conflicts.
- Regularly test contract functionality on a test network before deploying to production.

## 📖 References
- [Stellar Official Guide](https://developers.stellar.org/docs/)
- [Rust Book](https://doc.rust-lang.org/book/)

## 📌 Additional Notes
- The documentation will be updated as the contract evolves.
- Ensure all commands and examples are tested for accuracy before using them in a production environment.

