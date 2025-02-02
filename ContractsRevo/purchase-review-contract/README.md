# Purchase Review Contract

## Overview

The **Purchase Review Contract** is a smart contract designed to manage and verify purchase reviews in a decentralized way. It ensures that reviews are legitimate, associated with real transactions, and provides mechanisms to prevent fraudulent activity, enhancing trust between buyers and sellers.

## Functionality

### Features

- **Submitting Reviews**: Users can submit a review along with a purchase verification link. Each review is stored immutably and linked to a verified purchase.
- **Purchase Verification**: The contract verifies whether a user has purchased a product before allowing them to submit a review. If a purchase is not verified, the review submission is rejected.
- **Review Retrieval**: Users can query stored reviews for a specific product.
- **Rating System**: Users can submit ratings based on different categories, such as product quality, shipping experience, and customer service. Ratings use a weighted system to provide more accurate aggregations.
- **Voting System**: Other users can mark reviews as helpful or not helpful, influencing their visibility and credibility.
- **Reporting System**: Users can report reviews that violate platform rules, which can later be reviewed for potential removal.
- **Review Editing**: Users can edit their reviews within a specific time window (24 hours) after submission.

### Key Components

#### Data Structures

- `ReviewDetails`: Stores review text, timestamp, helpful votes, verification status, and responses.
- `ProductRatings`: Holds aggregated ratings for a product.
- `PurchaseVerificationData`: Links a user's purchase verification status with a product.
- `ReviewReportData`: Stores reports made against specific reviews.

#### Core Operations

- `submit_review`: Allows users to submit a review with a verification link.
- `get_review_details`: Retrieves a review's details by product ID and review ID.
- `vote_helpful`: Lets users mark a review as helpful or not.
- `report_review`: Enables users to report a review for violations.
- `is_review_editable`: Determines if a review can still be edited within the allowed timeframe.
- `verify_purchase`: Ensures that a purchase is valid before allowing review submission.

#### Error Handling

Various errors are implemented to prevent duplicate reviews, unauthorized access, invalid rating submissions, and excessive reporting attempts.

## Deployment

### Deploy Contract to Stellar Testnet

```sh
stellar contract deploy    --wasm ./target/wasm32-unknown-unknown/release/purchase_review.wasm    --source <source_account>    --network testnet
```

### Get Contract ID

After deployment, note the contract ID, which will be used for interactions.

## Interacting with the Contract

### Submit a Review

```sh
stellar contract invoke    --id <contract_id>    --source <user_account>    --network testnet    -- function submit_review    --args '{"product_id": "123", "review": "Great product!", "rating": 5}'
```

### Retrieve Reviews

```sh
stellar contract invoke    --id <contract_id>    --source <user_account>    --network testnet    -- function get_reviews    --args '{"product_id": "123"}'
```

### Verify a Review

```sh
stellar contract invoke    --id <contract_id>    --source <user_account>    --network testnet    -- function verify_review    --args '{"review_id": "456"}'
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
