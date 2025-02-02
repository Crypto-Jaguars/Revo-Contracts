# Purchase Review Contract

## 📌 Overview

The **Purchase Review Contract** is a smart contract designed to facilitate and verify purchase reviews in a decentralized manner. It ensures that reviews are legitimate and linked to actual transactions, preventing fraudulent activity and enhancing trust between buyers and sellers.

## 🛠 Prerequisites

Before using the contract, ensure you have the following installed:

- **Rust 🦀**: Used for writing and compiling the contract.
- **Stellar CLI 📡**: Required for deploying and interacting with the contract.
- **A Stellar Wallet 💳** (e.g., [Freighter](https://www.freighter.app/)): Needed for transactions.

## 📥 Directory Structure

### 1. Clone the Repository 🗂️

```bash
git clone https://github.com/<username>/Revo-Contracts.git

cd Revo-Contracts/purchase-review-contract/src
```

## 🔗 Compilation 

### 1. Build the Contract 🏗️

```bash
stellar contract build
```

### 2. Run Tests 🕵️

```bash
cargo test
```

## 🏗 Functionality

The contract allows users to:

- Submit a review after completing a purchase.
- Verify if a review is associated with a valid transaction.
- Prevent duplicate or fraudulent reviews.
- Provide immutable storage of reviews.
- Retrieve reviews for a given product.

## 🗝️ Key Components

#### Data Structures 📦

- `ReviewDetails`: Stores review text, timestamp, helpful votes, verification status, and responses.
- `ProductRatings`: Holds aggregated ratings for a product.
- `PurchaseVerificationData`: Links a user's purchase verification status with a product.
- `ReviewReportData`: Stores reports made against specific reviews.

#### Core Operations 🔏

- `submit_review`: Allows users to submit a review with a verification link.
- `get_review_details`: Retrieves a review's details by product ID and review ID.
- `vote_helpful`: Lets users mark a review as helpful or not.
- `report_review`: Enables users to report a review for violations.
- `is_review_editable`: Determines if a review can still be edited within the allowed timeframe.
- `verify_purchase`: Ensures that a purchase is valid before allowing review submission.

## 📂 Contract Code Implementation

### Data Structures 📦


```rust
#[contracttype]
#[derive(Clone)]
pub struct ReviewDetails {
    pub review_text: String,
    pub reviewer: Address,
    pub timestamp: u64,
    pub helpful_votes: u64,
    pub not_helpful_votes: u64,
    pub verified_purchase: bool,
    pub responses: Vec<String>,
}
```

### Core Operations 🔏

```rust
#[contractimpl]
impl PurchaseReviewContract {
    pub fn submit_review(
        env: Env,
        user: Address,
        product_id: u128,
        review_text: String,
        purchase_link: String,
    ) -> Result<(), PurchaseReviewError> {
        user.require_auth();
        if review_text.is_empty() || review_text.len() > 1000 {
            return Err(PurchaseReviewError::InvalidReviewText);
        }
        Self::verify_purchase(env.clone(), user.clone(), product_id, purchase_link)?;
        let review = ReviewDetails {
            review_text,
            reviewer: user.clone(),
            timestamp: env.ledger().timestamp(),
            helpful_votes: 0,
            not_helpful_votes: 0,
            verified_purchase: true,
            responses: Vec::new(&env),
        };
        env.storage().persistent().set(&(product_id, user.clone()), &review);
        Ok(())
    }

    pub fn get_review(env: Env, product_id: u128, user: Address) -> Result<ReviewDetails, PurchaseReviewError> {
        env.storage().persistent().get(&(product_id, user)).ok_or(PurchaseReviewError::ReviewNotFound)
    }
}
```

## 🚀 Deployment

### 1. Deploy Contract to Stellar Testnet

```bash
stellar contract deploy \
   --wasm ./target/wasm32-unknown-unknown/release/purchase_review.wasm \
   --source <source_account> \
   --network testnet
```

### 2. Get Contract ID 

After deployment, note the contract ID, which will be used for interactions.

## 📂 Interacting with the Contract

### 1. Submit a Review 📝

```bash
stellar contract invoke \
   --id <contract_id> \
   --source <user_account> \
   --network testnet \
   -- function submit_review \
   --args "{\"product_id\": \"123\", \"review\": \"Great product!\", \"rating\": 5}"
```

### 2. Retrieve Reviews 🔍

```bash
stellar contract invoke \
   --id <contract_id> \
   --source <user_account> \
   --network testnet \
   -- function get_reviews \
   --args "{\"product_id\": \"123\"}"
```

### 3. Verify a Review ✅

```bash
stellar contract invoke \
   --id <contract_id> \
   --source <user_account> \
   --network testnet \
   -- function verify_review \
   --args "{\"review_id\": \"456\"}"
```

## 🩺 Troubleshooting

### Common Issues & Fixes

1. **Rust installation errors**: Ensure `cargo` is in your system PATH.
2. **Stellar CLI not found**: Try running `stellar --help` to confirm installation.
3. **Wallet connectivity issues**: Verify that the correct network (testnet/mainnet) is configured.

---

### 📚 References

- [Stellar Smart Contract Guide](https://developers.stellar.org/#smart-contract-developers)
- [Rust Programming Book](https://doc.rust-lang.org/book/)

✅ **By following this guide, you should be able to deploy and interact with the Purchase Review Contract successfully.**