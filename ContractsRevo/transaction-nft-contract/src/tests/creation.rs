extern crate std;

use crate::tests::utils::*;
use soroban_sdk::{testutils::Events as _, BytesN, FromVal};

#[test]
fn test_nft_creation_success() {
    let (env, _contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

    // Verify the transaction ID is valid
    assert_eq!(tx_id.len(), 32, "Transaction ID should be 32 bytes");

    // Verify NFT metadata was stored correctly
    let metadata = client.get_nft_metadata(&tx_id).unwrap();
    verify_nft_metadata(&metadata, &buyer, &seller, amount, &product, 12345);
}

#[test]
fn test_unique_nft_ids() {
    let (env, _contract_id, client) = setup_test();

    // Create multiple transactions with different data
    let transactions = create_multiple_transactions(&env, 5);
    let mut nft_ids = std::vec::Vec::new();

    for (buyer, seller, amount, product) in transactions {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        nft_ids.push(tx_id);
    }

    // Verify all NFT IDs are unique
    for (i, id1) in nft_ids.iter().enumerate() {
        for (j, id2) in nft_ids.iter().enumerate() {
            if i != j {
                assert_ne!(id1, id2, "NFT IDs should be unique");
            }
        }
    }
}

#[test]
fn test_nft_creation_with_metadata_validation() {
    let (env, _contract_id, client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 5000_u64;
    let product = create_product_id(&env, 42);

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
    let metadata = client.get_nft_metadata(&tx_id).unwrap();

    // Validate all metadata fields
    assert_eq!(metadata.buyer, buyer, "Buyer address should match");
    assert_eq!(metadata.seller, seller, "Seller address should match");
    assert_eq!(metadata.amount, amount, "Amount should match");
    assert_eq!(metadata.product, product, "Product should match");
    assert!(metadata.timestamp > 0, "Timestamp should be positive");
    assert_eq!(
        metadata.timestamp, 12345,
        "Timestamp should match ledger time"
    );
}

#[test]
fn test_nft_creation_with_different_amounts() {
    let (env, _contract_id, client) = setup_test();

    let test_amounts = [1_u64, 100, 1000, 10000, u64::MAX];

    for &amount in &test_amounts {
        let (buyer, seller, _, product) = create_transaction_with_amount(&env, amount);
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

        let metadata = client.get_nft_metadata(&tx_id).unwrap();
        assert_eq!(
            metadata.amount, amount,
            "Amount should match for value: {}",
            amount
        );
    }
}

#[test]
fn test_nft_creation_with_different_products() {
    let (env, _contract_id, client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 1000_u64;

    let products = create_product_variations(&env);

    for product in products {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        let metadata = client.get_nft_metadata(&tx_id).unwrap();

        assert_eq!(
            metadata.product, product,
            "Product should match in metadata"
        );
        assert_eq!(metadata.amount, amount, "Amount should remain consistent");
    }
}

#[test]
fn test_nft_creation_event_emission() {
    let (env, _contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

    // Check that the minting event was emitted
    let events = env.events().all();
    let nft_minted_event_exists = events.iter().any(|(_, _, value)| {
        let value_as_bytes = BytesN::<32>::from_val(&env, &value);
        value_as_bytes == tx_id
    });

    assert!(
        nft_minted_event_exists,
        "NFT minting event should be emitted"
    );
}

#[test]
fn test_nft_creation_timestamp_accuracy() {
    let timestamp = 98765_u64;
    let (env, _contract_id, client) = setup_test_with_timestamp(timestamp);
    let (buyer, seller, amount, product) = create_standard_transaction(&env);

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
    let metadata = client.get_nft_metadata(&tx_id).unwrap();

    assert_eq!(
        metadata.timestamp, timestamp,
        "Timestamp should match ledger timestamp"
    );
}

#[test]
fn test_nft_creation_with_time_progression() {
    let (env, _contract_id, client) = setup_test();
    let (buyer, seller, amount, product1) = create_standard_transaction(&env);

    // Create first NFT
    let tx_id1 = client.mint_nft(&buyer, &seller, &amount, &product1);
    let metadata1 = client.get_nft_metadata(&tx_id1).unwrap();

    // Advance time
    advance_time(&env, 3600); // 1 hour

    // Create second NFT with different product
    let product2 = create_product_id(&env, 2);
    let tx_id2 = client.mint_nft(&buyer, &seller, &amount, &product2);
    let metadata2 = client.get_nft_metadata(&tx_id2).unwrap();

    assert!(
        metadata2.timestamp > metadata1.timestamp,
        "Second NFT should have later timestamp"
    );
    assert_ne!(
        tx_id1, tx_id2,
        "NFTs should have different IDs even with same parties"
    );
}

#[test]
#[should_panic(expected = "Buyer and seller cannot be the same address")]
fn test_nft_creation_same_buyer_seller() {
    let (env, _contract_id, client) = setup_test();
    let address = create_buyer(&env);
    let amount = 1000_u64;
    let product = create_product_id(&env, 1);

    // This should panic because buyer and seller are the same
    client.mint_nft(&address, &address, &amount, &product);
}

#[test]
#[should_panic(expected = "Amount must be greater than zero")]
fn test_nft_creation_zero_amount() {
    let (env, _contract_id, client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 0_u64;
    let product = create_product_id(&env, 1);

    // This should panic because amount is zero
    client.mint_nft(&buyer, &seller, &amount, &product);
}

#[test]
#[should_panic(expected = "Duplicate transaction detected")]
fn test_nft_creation_duplicate_transaction() {
    let (env, _contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);

    // Create the first NFT
    client.mint_nft(&buyer, &seller, &amount, &product);

    // Attempting to create the same transaction again should panic
    client.mint_nft(&buyer, &seller, &amount, &product);
}

#[test]
#[should_panic(expected = "Invalid ledger timestamp")]
fn test_nft_creation_with_invalid_timestamp_environment() {
    let (env, _contract_id, client) = setup_test_with_timestamp(0);
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 1000_u64;
    let product = create_product_id(&env, 1);

    // This should panic because timestamp is 0
    client.mint_nft(&buyer, &seller, &amount, &product);
}

#[test]
fn test_nft_creation_different_buyer_seller_combinations() {
    let (env, _contract_id, client) = setup_test();
    let addresses = create_multiple_addresses(&env, 4);
    let amount = 1000_u64;

    let mut nft_ids = std::vec::Vec::new();
    let mut combination_count = 0;

    // Create NFTs for different buyer-seller combinations
    for (i, buyer) in addresses.iter().enumerate() {
        for (j, seller) in addresses.iter().enumerate() {
            if i != j {
                // Ensure buyer != seller
                let product = create_product_id(&env, combination_count);
                let tx_id = client.mint_nft(buyer, seller, &amount, &product);
                nft_ids.push(tx_id);
                combination_count += 1;
            }
        }
    }

    // Verify all NFTs are unique
    for (i, id1) in nft_ids.iter().enumerate() {
        for (j, id2) in nft_ids.iter().enumerate() {
            if i != j {
                assert_ne!(id1, id2, "All NFT IDs should be unique");
            }
        }
    }

    assert_eq!(
        nft_ids.len(),
        12,
        "Should have created 12 unique NFTs (4*3 combinations)"
    );
}

#[test]
fn test_nft_metadata_retrieval_nonexistent() {
    let (_env, _contract_id, client) = setup_test();
    let fake_tx_id = BytesN::from_array(&_env, &[99; 32]);

    let metadata = client.get_nft_metadata(&fake_tx_id);
    assert!(
        metadata.is_none(),
        "Should return None for non-existent NFT"
    );
}

#[test]
fn test_nft_creation_metadata_consistency() {
    let (env, _contract_id, client) = setup_test();
    let transactions = create_multiple_transactions(&env, 10);

    for (buyer, seller, amount, product) in transactions {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        let metadata = client.get_nft_metadata(&tx_id).unwrap();

        // Verify metadata consistency
        verify_nft_metadata(&metadata, &buyer, &seller, amount, &product, 12345);

        // Verify the metadata can be retrieved multiple times consistently
        let metadata2 = client.get_nft_metadata(&tx_id).unwrap();
        assert_eq!(
            metadata.buyer, metadata2.buyer,
            "Buyer should be consistent"
        );
        assert_eq!(
            metadata.seller, metadata2.seller,
            "Seller should be consistent"
        );
        assert_eq!(
            metadata.amount, metadata2.amount,
            "Amount should be consistent"
        );
        assert_eq!(
            metadata.product, metadata2.product,
            "Product should be consistent"
        );
        assert_eq!(
            metadata.timestamp, metadata2.timestamp,
            "Timestamp should be consistent"
        );
    }
}

#[test]
fn test_nft_creation_large_amounts() {
    let (env, _contract_id, client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let product = create_product_id(&env, 1);

    // Test with very large amounts
    let large_amounts = [u64::MAX - 1, u64::MAX / 2, 1_000_000_000_000_u64];

    for &amount in &large_amounts {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        let metadata = client.get_nft_metadata(&tx_id).unwrap();

        assert_eq!(
            metadata.amount, amount,
            "Large amount should be stored correctly"
        );
    }
}

#[test]
fn test_nft_creation_edge_case_products() {
    let (env, _contract_id, client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 1000_u64;

    // Test edge case products
    let edge_products = [
        create_product_id(&env, 0),   // All zeros
        create_product_id(&env, 255), // All 255s
        create_product_id(&env, 128), // Mid-range
    ];

    for product in edge_products {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        let metadata = client.get_nft_metadata(&tx_id).unwrap();

        assert_eq!(
            metadata.product, product,
            "Edge case product should be stored correctly"
        );
    }
}
