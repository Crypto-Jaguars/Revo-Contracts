<# Transaction NFT Contract

## 📌 Overview
The **Transaction NFT Contract** is a smart contract designed for secure and verifiable NFT transactions on the blockchain. It facilitates transparent ownership transfers and seamless interactions with decentralized applications (DApps).

## 🚀 Features
- **NFT Minting & Ownership Management** 🖼️
- **Secure and Transparent Transactions** 🔗
- **On-Chain Metadata Storage** 📄
- **Interoperability with Stellar Accounts** 🚀

## 🛠 Prerequisites
Before using the contract, ensure you have:
- [Rust](https://www.rust-lang.org/)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)
- [Stellar SDK](https://developers.stellar.org/)

## 🔧 Setup & Deployment

### Clone Repository
```sh
git clone https://github.com/Crypto-Jaguars/Revo-Contracts.git
cd Revo-Contracts/transaction-nft-contract
```

### Build Contract
```sh
stellar contract build
```

### Deploy Contract
```sh
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/transaction_nft_contract.wasm
```

## 🔄 Usage

### Mint an NFT
```sh
soroban contract invoke --id <CONTRACT_ID> --fn mint --arg <NFT_METADATA>
```

### Transfer an NFT
```sh
soroban contract invoke --id <CONTRACT_ID> --fn transfer --arg <TO_ADDRESS>
```

## 🧪 Testing
Run tests to verify contract functionality:
```sh
cargo test
```

## 📖 References
- [Stellar Soroban Guide](https://soroban.stellar.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
