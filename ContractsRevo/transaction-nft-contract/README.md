# Transaction NFT Contract

## 📌 Overview
The **Transaction NFT Contract** is a smart contract built on the Soroban framework for the Stellar blockchain. It creates non-fungible tokens (NFTs) that represent agricultural product transactions between buyers and sellers. Each NFT serves as an immutable record of a transaction, providing proof of purchase, ownership, and transaction details that can be verified on-chain.

## 🚀 Features
- **Transaction Verification**: Creates tamper-proof records of agricultural product transactions
- **Dual Authentication**: Requires authorization from both buyer and seller
- **Duplicate Prevention**: Prevents creation of duplicate transaction records
- **Metadata Storage**: Stores comprehensive transaction details on-chain
- **Proof Generation**: Creates cryptographic proofs of transactions
- **Queryable Records**: Allows retrieval of transaction metadata by ID

## 🛠 Contract Functionality

### 1. NFT Minting
The `mint_nft` function creates a new transaction NFT with the following steps:
- Validates that buyer and seller are different addresses
- Ensures transaction amount is greater than zero
- Requires authorization from both buyer and seller
- Checks for duplicate transactions
- Generates a unique transaction ID
- Creates and stores NFT metadata
- Emits an event for tracking

### 2. Metadata Retrieval
The `get_nft_metadata` function allows anyone to retrieve the metadata associated with a transaction NFT, including:
- Buyer address
- Seller address
- Transaction amount
- Product identifier
- Transaction timestamp

### 3. Transaction Proof
The contract generates cryptographic proofs of transactions that can be verified on-chain, ensuring the integrity and authenticity of transaction records.

## 📊 Data Structures

### NFTMetadata
```rust
pub struct NFTMetadata {
    pub buyer: Address,
    pub seller: Address,
    pub amount: u64,
    pub product: BytesN<32>,
    pub timestamp: u64,
}
```

## 🔧 Setup & Deployment

### Prerequisites
Before using the contract, ensure you have:
- [Rust](https://www.rust-lang.org/)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)
- [Stellar SDK](https://developers.stellar.org/)

### Installation Steps
1. **Clone the Repository**
   ```bash
   git clone https://github.com/Crypto-Jaguars/Revo-Contracts.git
   cd Revo-Contracts/transaction-nft-contract
   ```
2. **Build the Contract**
   ```bash
   stellar contract build
   ```
3. **Run the Tests**
   ```bash
   cargo test
   ```

### Deployment
```bash
stellar contract deploy
```

## 🔄 Usage Examples

### Mint a Transaction NFT
```bash
stellar contract invoke --id $CONTRACT_ID --fn mint_nft \
  --arg $BUYER_ADDRESS \
  --arg $SELLER_ADDRESS \
  --arg $AMOUNT \
  --arg $PRODUCT_ID
```

### Retrieve NFT Metadata
```bash
stellar contract invoke --id $CONTRACT_ID --fn get_nft_metadata \
  --arg $TRANSACTION_ID
```

## 📌 Best Practices
- Always ensure both buyer and seller authorize the transaction
- Store the returned transaction ID for future reference
- Verify transaction metadata after minting
- Use unique product identifiers to prevent duplicates

## 🔒 Security Features
- Dual authorization requirements
- Duplicate transaction prevention
- Immutable transaction records
- Cryptographic proof generation

## 🌐 Use Cases
- Verifiable proof of agricultural product purchases
- Supply chain transparency and traceability
- Digital receipts for transactions
- Integration with agricultural marketplaces
- Proof of ownership for premium or specialty products

## 📖 References
- [Stellar Documentation](https://developers.stellar.org/docs)
- [Soroban Documentation](https://soroban.stellar.org/)
- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Stellar Soroban Guide](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
