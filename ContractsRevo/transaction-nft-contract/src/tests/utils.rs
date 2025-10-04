extern crate std;

use crate::{TransactionNFTContract, TransactionNFTContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Bytes, BytesN, Env,
};

/// Create a standardized test environment with mock auths and unlimited budget
pub fn setup_test() -> (Env, Address, TransactionNFTContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    // Set a consistent ledger timestamp for testing
    env.ledger().set(create_ledger_info(12345));

    let contract_id = env.register(TransactionNFTContract, ());
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    (env, contract_id, client)
}

/// Create a test environment with a specific timestamp
pub fn setup_test_with_timestamp(
    timestamp: u64,
) -> (Env, Address, TransactionNFTContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    env.ledger().set(create_ledger_info(timestamp));

    let contract_id = env.register(TransactionNFTContract, ());
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    (env, contract_id, client)
}

/// Create standardized ledger info for consistent testing
pub fn create_ledger_info(timestamp: u64) -> LedgerInfo {
    LedgerInfo {
        timestamp,
        protocol_version: 22,
        sequence_number: 10,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    }
}

/// Generate a unique buyer address for testing
pub fn create_buyer(env: &Env) -> Address {
    Address::generate(env)
}

/// Generate a unique seller address for testing
pub fn create_seller(env: &Env) -> Address {
    Address::generate(env)
}

/// Create a standard product BytesN for testing
pub fn create_product_id(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// Create a standard product Bytes for testing (used in proof functions)
pub fn create_product_bytes(env: &Env, seed: u8) -> Bytes {
    Bytes::from_array(env, &[seed; 32])
}

/// Generate multiple unique addresses for bulk testing
pub fn create_multiple_addresses(env: &Env, count: usize) -> std::vec::Vec<Address> {
    (0..count).map(|_| Address::generate(env)).collect()
}

/// Create a valid transaction for testing
pub fn create_standard_transaction(env: &Env) -> (Address, Address, u64, BytesN<32>) {
    let buyer = create_buyer(env);
    let seller = create_seller(env);
    let amount = 1000_u64;
    let product = create_product_id(env, 1);

    (buyer, seller, amount, product)
}

/// Create multiple standard transactions for bulk testing
pub fn create_multiple_transactions(
    env: &Env,
    count: usize,
) -> std::vec::Vec<(Address, Address, u64, BytesN<32>)> {
    (0..count)
        .map(|i| {
            let buyer = create_buyer(env);
            let seller = create_seller(env);
            let amount = 1000_u64 + i as u64;
            let product = create_product_id(env, (i + 1) as u8);
            (buyer, seller, amount, product)
        })
        .collect()
}

/// Advance ledger time for testing time-based functionality
pub fn advance_time(env: &Env, seconds: u64) {
    let current_info = env.ledger().get();
    let new_timestamp = current_info.timestamp + seconds;
    let mut new_info = current_info.clone();
    new_info.timestamp = new_timestamp;
    new_info.sequence_number += 1;
    env.ledger().set(new_info);
}

/// Create invalid transaction data for error testing
pub fn create_invalid_transaction_data(env: &Env) -> (Address, Address, u64, BytesN<32>) {
    let buyer = create_buyer(env);
    let seller = buyer.clone(); // Same address - invalid
    let amount = 0_u64; // Zero amount - invalid
    let product = create_product_id(env, 255);

    (buyer, seller, amount, product)
}

/// Verify NFT metadata matches expected transaction data
pub fn verify_nft_metadata(
    metadata: &crate::mint::NFTMetadata,
    expected_buyer: &Address,
    expected_seller: &Address,
    expected_amount: u64,
    expected_product: &BytesN<32>,
    expected_min_timestamp: u64,
) {
    assert_eq!(
        &metadata.buyer, expected_buyer,
        "Buyer mismatch in NFT metadata"
    );
    assert_eq!(
        &metadata.seller, expected_seller,
        "Seller mismatch in NFT metadata"
    );
    assert_eq!(
        metadata.amount, expected_amount,
        "Amount mismatch in NFT metadata"
    );
    assert_eq!(
        &metadata.product, expected_product,
        "Product mismatch in NFT metadata"
    );
    assert!(
        metadata.timestamp >= expected_min_timestamp,
        "Timestamp should be at least the expected minimum"
    );
}

/// Create transaction with specific amount for price testing
pub fn create_transaction_with_amount(
    env: &Env,
    amount: u64,
) -> (Address, Address, u64, BytesN<32>) {
    let buyer = create_buyer(env);
    let seller = create_seller(env);
    let product = create_product_id(env, 1);

    (buyer, seller, amount, product)
}

/// Helper to create large-volume test data
pub fn create_high_volume_test_data(
    env: &Env,
    volume: usize,
) -> std::vec::Vec<(Address, Address, u64, BytesN<32>)> {
    (0..volume)
        .map(|i| {
            let buyer = create_buyer(env);
            let seller = create_seller(env);
            let amount = (i + 1) as u64 * 100;
            let product = create_product_id(env, (i % 255) as u8);
            (buyer, seller, amount, product)
        })
        .collect()
}

/// Verify transaction proof was created successfully
pub fn verify_transaction_proof_exists(
    env: &Env,
    contract_id: &Address,
    buyer: &Address,
    seller: &Address,
    amount: u64,
    product: &Bytes,
) -> bool {
    env.as_contract(contract_id, || {
        crate::proof::transaction_exists(env, buyer, seller, amount, product)
    })
}

/// Create different product variations for uniqueness testing
pub fn create_product_variations(env: &Env) -> std::vec::Vec<BytesN<32>> {
    std::vec![
        create_product_id(env, 1),   // Standard product
        create_product_id(env, 2),   // Different product
        create_product_id(env, 255), // Max value product
        create_product_id(env, 128), // Mid-range product
        create_product_id(env, 0),   // Min value product
    ]
}
