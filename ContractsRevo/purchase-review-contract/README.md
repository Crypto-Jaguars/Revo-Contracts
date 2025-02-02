# Purchase Review Contract

## 📌 Overview

The **Purchase Review Contract** is a smart contract designed to facilitate and verify purchase reviews in a decentralized manner. It ensures that reviews are legitimate and linked to actual transactions, preventing fraudulent activity and enhancing trust between buyers and sellers.

## 🏗 Functionality

The contract allows users to:

- Submit a review after completing a purchase.
- Verify if a review is associated with a valid transaction.
- Prevent duplicate or fraudulent reviews.
- Provide immutable storage of reviews.
- Retrieve reviews for a given product.

## 🛠 Prerequisites

Before using the contract, ensure you have the following installed:

- **Rust 🦀**: Used for writing and compiling the contract.
- **Stellar CLI 📡**: Required for deploying and interacting with the contract.
- **A Stellar Wallet 💳** (e.g., [Freighter](https://www.freighter.app/)): Needed for transactions.

## 📥 Installation and Setup

### 1. Install Rust 🦀

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

### 2. Install Stellar CLI 📡

```bash
cargo install --locked stellar-cli --features opt
```

### 3. Clone the Repository 🗂️

```bash
git clone https://github.com/<username>/purchase-review-contract.git
cd purchase-review-contract
```

### 4. Build the Contract 🏗️

```bash
stellar contract build
```

### 5. Run Tests 🕵️

```bash
cargo test
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

## 🔗 Interacting with the Contract

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
