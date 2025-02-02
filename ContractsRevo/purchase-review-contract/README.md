# Purchase Review Contract

## Overview

The **Purchase Review Contract** is a smart contract designed to manage and verify purchase reviews in a decentralized manner. It ensures that reviews are legitimate, associated with real transactions, and provides mechanisms to prevent fraudulent activity, enhancing trust between buyers and sellers.

## Functionality

### Features

- **Submitting Reviews**: Users can submit a review along with a purchase verification link. Each review is stored immutably and linked to a verified purchase.
- **Purchase Verification**: The contract verifies whether a user has purchased a product before allowing them to submit a review. If a purchase is not verified, the review submission is rejected.
- **Review Retrieval**: Users can query stored reviews for a specific product.
- **Rating System**: Users can submit ratings based on different categories, such as product quality, shipping experience, and customer service. Ratings use a weighted system to provide more accurate aggregations.
- **Voting System**: Other users can mark reviews as helpful or not helpful, influencing their visibility and credibility.
- **Reporting System**: Users can report reviews that violate platform rules, which can later be reviewed for potential removal.
- **Review Editing**: Users can edit their reviews within a specific time window (24 hours) after submission.

## Contract Code Implementation

### Data Structures

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

### Core Operations

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

### Deployment

### Deploy Contract to Stellar Testnet

```sh
stellar contract deploy --wasm ./target/wasm32-unknown-unknown/release/purchase_review.wasm --source <source_account> --network testnet
```

### Get Contract ID

After deployment, note the contract ID, which will be used for interactions.

## Interacting with the Contract

### Submit a Review

```sh
stellar contract invoke --id <contract_id> --source <user_account> --network testnet -- function submit_review --args '{"product_id": "123", "review": "Great product!", "rating": 5}'
```

### Retrieve Reviews

```sh
stellar contract invoke --id <contract_id> --source <user_account> --network testnet -- function get_reviews --args '{"product_id": "123"}'
```

### Verify a Review

```sh
stellar contract invoke --id <contract_id> --source <user_account> --network testnet -- function verify_review --args '{"review_id": "456"}'
```

## Troubleshooting

### Common Issues & Fixes

1. **Rust installation errors**: Ensure `cargo` is in your system PATH.
2. **Stellar CLI not found**: Try running `stellar --help` to confirm installation.
3. **Wallet connectivity issues**: Verify that the correct network (testnet/mainnet) is configured.

## References

- [Stellar Smart Contract Guide](https://developers.stellar.org/#smart-contract-developers)
- [Rust Programming Book](https://doc.rust-lang.org/book/)

By following this guide, you should be able to deploy and interact with the Purchase Review Contract successfully.
