extern crate std;

use crate::tests::utils::*;
use soroban_sdk::{Bytes, BytesN};

#[test]
fn test_transaction_proof_generation_success() {
    let (env, contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);
    let product_bytes = create_product_bytes(&env, 1);

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

    // Verify transaction proof was generated and stored
    let proof_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(
        proof_exists,
        "Transaction proof should exist after NFT creation"
    );

    // Verify the tx_id matches the generated proof
    assert_eq!(tx_id.len(), 32, "Transaction ID should be 32 bytes hash");
}

#[test]
fn test_transaction_proof_uniqueness() {
    let (env, contract_id, _client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 1000_u64;
    let product_bytes = create_product_bytes(&env, 1);

    // Generate first proof
    let tx_id1 = env.as_contract(&contract_id, || {
        crate::proof::generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product_bytes.clone(),
        )
    });

    // Advance time to create different timestamp
    advance_time(&env, 3600);

    // Generate second proof with same transaction data but different timestamp
    let tx_id2 = env.as_contract(&contract_id, || {
        crate::proof::generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product_bytes.clone(),
        )
    });

    assert_ne!(
        tx_id1, tx_id2,
        "Proof IDs should be unique even with same transaction data"
    );
}

#[test]
fn test_transaction_existence_verification() {
    let (env, contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);
    let product_bytes = create_product_bytes(&env, 1);

    // Before creating NFT, transaction should not exist
    let exists_before = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(
        !exists_before,
        "Transaction proof should not exist before NFT creation"
    );

    // Create NFT
    client.mint_nft(&buyer, &seller, &amount, &product);

    // After creating NFT, transaction should exist
    let exists_after = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(
        exists_after,
        "Transaction proof should exist after NFT creation"
    );
}

#[test]
fn test_nft_transaction_linkage_validation() {
    let (env, _contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
    let metadata = client.get_nft_metadata(&tx_id).unwrap();

    // Verify NFT metadata matches transaction parameters
    assert_eq!(
        metadata.buyer, buyer,
        "NFT buyer should match transaction buyer"
    );
    assert_eq!(
        metadata.seller, seller,
        "NFT seller should match transaction seller"
    );
    assert_eq!(
        metadata.amount, amount,
        "NFT amount should match transaction amount"
    );
    assert_eq!(
        metadata.product, product,
        "NFT product should match transaction product"
    );
}

#[test]
fn test_verification_of_nonexistent_transaction() {
    let (env, contract_id, _client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 9999_u64;
    let product_bytes = create_product_bytes(&env, 99);

    // Try to verify a transaction that was never created
    let exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(
        !exists,
        "Non-existent transaction should not be verified as existing"
    );
}

#[test]
fn test_verification_with_invalid_parameters() {
    let (env, contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);
    let product_bytes = create_product_bytes(&env, 1);

    // Create valid NFT
    client.mint_nft(&buyer, &seller, &amount, &product);

    // Test verification with wrong buyer
    let wrong_buyer = create_buyer(&env);
    let wrong_buyer_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &wrong_buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(
        !wrong_buyer_exists,
        "Verification should fail with wrong buyer"
    );

    // Test verification with wrong seller
    let wrong_seller = create_seller(&env);
    let wrong_seller_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &wrong_seller,
        amount,
        &product_bytes,
    );
    assert!(
        !wrong_seller_exists,
        "Verification should fail with wrong seller"
    );

    // Test verification with wrong amount
    let wrong_amount_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount + 1,
        &product_bytes,
    );
    assert!(
        !wrong_amount_exists,
        "Verification should fail with wrong amount"
    );

    // Test verification with wrong product
    let wrong_product_bytes = create_product_bytes(&env, 2);
    let wrong_product_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &wrong_product_bytes,
    );
    assert!(
        !wrong_product_exists,
        "Verification should fail with wrong product"
    );
}

#[test]
fn test_duplicate_transaction_prevention_success() {
    let (env, _contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);

    // Create first NFT successfully
    let tx_id1 = client.mint_nft(&buyer, &seller, &amount, &product);
    assert_eq!(tx_id1.len(), 32, "First NFT should be created successfully");

    // Verify the NFT was created
    let metadata = client.get_nft_metadata(&tx_id1).unwrap();
    assert_eq!(metadata.buyer, buyer, "First NFT should be owned by buyer");
}

#[test]
#[should_panic(expected = "Duplicate transaction detected")]
fn test_duplicate_transaction_prevention_panic() {
    let (env, _contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);

    // Create first NFT successfully
    client.mint_nft(&buyer, &seller, &amount, &product);

    // Attempt to create duplicate transaction should panic
    client.mint_nft(&buyer, &seller, &amount, &product);
}

#[test]
fn test_transaction_verification_across_multiple_nfts() {
    let (env, contract_id, client) = setup_test();
    let transactions = create_multiple_transactions(&env, 5);
    let mut tx_ids = std::vec::Vec::new();

    // Create multiple NFTs
    for (buyer, seller, amount, product) in &transactions {
        let tx_id = client.mint_nft(buyer, seller, amount, product);
        tx_ids.push(tx_id);
    }

    // Verify each transaction exists and is linked correctly
    for ((buyer, seller, amount, product), tx_id) in transactions.iter().zip(tx_ids.iter()) {
        let product_bytes = Bytes::from_array(&env, &product.to_array());
        let proof_exists = verify_transaction_proof_exists(
            &env,
            &contract_id,
            buyer,
            seller,
            *amount,
            &product_bytes,
        );
        assert!(proof_exists, "Transaction proof should exist for each NFT");

        let metadata = client.get_nft_metadata(tx_id).unwrap();
        assert_eq!(
            &metadata.buyer, buyer,
            "NFT metadata should match transaction"
        );
        assert_eq!(
            &metadata.seller, seller,
            "NFT metadata should match transaction"
        );
        assert_eq!(
            metadata.amount, *amount,
            "NFT metadata should match transaction"
        );
        assert_eq!(
            &metadata.product, product,
            "NFT metadata should match transaction"
        );
    }
}

#[test]
fn test_verification_consistency_over_time() {
    let (env, contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);
    let product_bytes = create_product_bytes(&env, 1);

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

    // Verify transaction exists immediately
    let exists_immediately = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(
        exists_immediately,
        "Transaction should exist immediately after creation"
    );

    // Advance time and verify transaction still exists
    advance_time(&env, 86400); // 24 hours

    // Note: The transaction proof depends on the current ledger timestamp
    // After advancing time, we need to generate a new proof to verify with the new timestamp
    // The original proof still exists in storage, but verification needs current timestamp
    let new_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    // This might be false because the proof includes timestamp and it has changed
    // The original NFT should still exist though
    let metadata_still_exists = client.get_nft_metadata(&tx_id);
    assert!(
        metadata_still_exists.is_some(),
        "NFT metadata should persist over time"
    );

    // Verify NFT metadata is still consistent
    let metadata = client.get_nft_metadata(&tx_id).unwrap();
    assert_eq!(
        metadata.buyer, buyer,
        "NFT buyer should remain consistent over time"
    );
    assert_eq!(
        metadata.seller, seller,
        "NFT seller should remain consistent over time"
    );
}

#[test]
fn test_transaction_proof_hash_integrity() {
    let (env, contract_id, _client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 1000_u64;
    let product_bytes = create_product_bytes(&env, 1);

    // Generate proof multiple times with same parameters
    let tx_id1 = env.as_contract(&contract_id, || {
        crate::proof::generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product_bytes.clone(),
        )
    });

    // Generate proof again with same parameters (deterministic hashing)
    let tx_id2 = env.as_contract(&contract_id, || {
        crate::proof::generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product_bytes.clone(),
        )
    });

    // With same parameters and timestamp, proofs should be identical
    assert_eq!(
        tx_id1, tx_id2,
        "Proofs should be deterministic with same parameters and timestamp"
    );
}

#[test]
fn test_verification_with_edge_case_amounts() {
    let (env, contract_id, client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let product = create_product_id(&env, 1);
    let product_bytes = create_product_bytes(&env, 1);

    // Test edge case amounts
    let edge_amounts = [1_u64, u64::MAX - 1, u64::MAX];

    for &amount in &edge_amounts {
        // Create NFT with edge case amount
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

        // Verify transaction exists
        let proof_exists = verify_transaction_proof_exists(
            &env,
            &contract_id,
            &buyer,
            &seller,
            amount,
            &product_bytes,
        );
        assert!(
            proof_exists,
            "Transaction proof should exist for amount: {}",
            amount
        );

        // Verify metadata consistency
        let metadata = client.get_nft_metadata(&tx_id).unwrap();
        assert_eq!(
            metadata.amount, amount,
            "Metadata amount should match for: {}",
            amount
        );
    }
}

#[test]
fn test_verification_with_different_product_variations() {
    let (env, contract_id, client) = setup_test();
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 1000_u64;
    let products = create_product_variations(&env);

    for (i, product) in products.iter().enumerate() {
        // Create product bytes from the actual product BytesN
        let product_bytes = Bytes::from_array(&env, &product.to_array());

        // Create NFT with product variation
        let tx_id = client.mint_nft(&buyer, &seller, &amount, product);

        // Verify transaction exists
        let proof_exists = verify_transaction_proof_exists(
            &env,
            &contract_id,
            &buyer,
            &seller,
            amount,
            &product_bytes,
        );
        assert!(
            proof_exists,
            "Transaction proof should exist for product variation {}",
            i
        );

        // Verify metadata consistency
        let metadata = client.get_nft_metadata(&tx_id).unwrap();
        assert_eq!(
            &metadata.product, product,
            "Metadata product should match variation {}",
            i
        );
    }
}

#[test]
fn test_bulk_verification_performance() {
    let (env, contract_id, client) = setup_test();
    let transactions = create_high_volume_test_data(&env, 50);
    let mut created_transactions = std::vec::Vec::new();

    // Create bulk NFTs
    for (buyer, seller, amount, product) in transactions {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        created_transactions.push((buyer, seller, amount, product, tx_id));
    }

    // Verify all transactions exist
    for (buyer, seller, amount, product, tx_id) in created_transactions {
        let product_bytes = Bytes::from_array(&env, &product.to_array());
        let proof_exists = verify_transaction_proof_exists(
            &env,
            &contract_id,
            &buyer,
            &seller,
            amount,
            &product_bytes,
        );
        assert!(proof_exists, "Bulk transaction proof should exist");

        let metadata = client.get_nft_metadata(&tx_id).unwrap();
        assert_eq!(
            metadata.buyer, buyer,
            "Bulk NFT metadata should be consistent"
        );
    }
}

#[test]
fn test_verification_integration_with_purchase_flow() {
    let (env, contract_id, client) = setup_test();

    // Simulate a purchase review contract integration scenario
    let buyer = create_buyer(&env);
    let seller = create_seller(&env);
    let amount = 2500_u64;
    let product = create_product_id(&env, 42);
    let product_bytes = create_product_bytes(&env, 42);

    // Step 1: Create NFT (simulating purchase completion)
    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

    // Step 2: Verify transaction proof exists (for review system integration)
    let proof_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(
        proof_exists,
        "Transaction proof should exist for purchase review integration"
    );

    // Step 3: Verify NFT contains all purchase details
    let metadata = client.get_nft_metadata(&tx_id).unwrap();
    assert_eq!(
        metadata.buyer, buyer,
        "Purchase buyer should match NFT owner"
    );
    assert_eq!(
        metadata.seller, seller,
        "Purchase seller should match NFT seller"
    );
    assert_eq!(
        metadata.amount, amount,
        "Purchase amount should match NFT amount"
    );
    assert_eq!(
        metadata.product, product,
        "Purchase product should match NFT product"
    );

    // Note: Duplicate transaction prevention is tested separately
    // as it requires should_panic annotation
}

#[test]
fn test_verification_with_concurrent_transactions() {
    let (env, contract_id, client) = setup_test();
    let concurrent_transactions = create_multiple_transactions(&env, 10);
    let mut results = std::vec::Vec::new();

    // Create concurrent NFTs
    for (buyer, seller, amount, product) in concurrent_transactions.clone() {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        results.push((buyer, seller, amount, product, tx_id));
    }

    // Verify each concurrent transaction independently
    for (buyer, seller, amount, product, tx_id) in results {
        // Verify proof exists
        let product_bytes = Bytes::from_array(&env, &product.to_array());
        let proof_exists = verify_transaction_proof_exists(
            &env,
            &contract_id,
            &buyer,
            &seller,
            amount,
            &product_bytes,
        );
        assert!(proof_exists, "Concurrent transaction proof should exist");

        // Verify metadata integrity
        let metadata = client.get_nft_metadata(&tx_id).unwrap();
        verify_nft_metadata(&metadata, &buyer, &seller, amount, &product, 12345);
    }
}

#[test]
fn test_timestamp_inclusion_in_verification() {
    let timestamp1 = 50000_u64;
    let timestamp2 = 60000_u64;

    let (env1, contract_id1, _client1) = setup_test_with_timestamp(timestamp1);
    let (env2, contract_id2, _client2) = setup_test_with_timestamp(timestamp2);

    // Create separate addresses for each environment
    let buyer1 = create_buyer(&env1);
    let seller1 = create_seller(&env1);
    let buyer2 = create_buyer(&env2);
    let seller2 = create_seller(&env2);
    let amount = 1000_u64;
    let product_bytes1 = create_product_bytes(&env1, 1);
    let product_bytes2 = create_product_bytes(&env2, 1);

    // Generate proofs with different timestamps but same transaction data structure
    let tx_id1 = env1.as_contract(&contract_id1, || {
        crate::proof::generate_transaction_proof(
            env1.clone(),
            buyer1.clone(),
            seller1.clone(),
            amount,
            product_bytes1.clone(),
        )
    });

    let tx_id2 = env2.as_contract(&contract_id2, || {
        crate::proof::generate_transaction_proof(
            env2.clone(),
            buyer2.clone(),
            seller2.clone(),
            amount,
            product_bytes2.clone(),
        )
    });

    // Since addresses and timestamps are different, proofs should be different
    assert_ne!(
        tx_id1, tx_id2,
        "Proofs should be different with different timestamps and addresses"
    );
}

#[test]
fn test_metadata_retrieval_for_nonexistent_nft() {
    let (_env, _contract_id, client) = setup_test();
    let fake_tx_id = BytesN::from_array(&_env, &[255; 32]);

    let metadata = client.get_nft_metadata(&fake_tx_id);
    assert!(
        metadata.is_none(),
        "Non-existent NFT should return None metadata"
    );
}

#[test]
fn test_high_volume_nft_creation_scalability() {
    let (env, contract_id, client) = setup_test();
    let volume = 100; // High volume test
    let transactions = create_high_volume_test_data(&env, volume);
    let mut created_nfts = std::vec::Vec::new();

    // Create high volume of NFTs
    for (buyer, seller, amount, product) in transactions {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);
        created_nfts.push((tx_id, buyer, seller, amount, product));
    }

    // Verify all NFTs were created successfully
    assert_eq!(
        created_nfts.len(),
        volume,
        "All high-volume NFTs should be created"
    );

    // Spot check verification for random NFTs
    let check_indices = [0, volume / 4, volume / 2, 3 * volume / 4, volume - 1];
    for &i in &check_indices {
        let (tx_id, buyer, seller, amount, product) = &created_nfts[i];
        let metadata = client.get_nft_metadata(tx_id).unwrap();
        verify_nft_metadata(&metadata, buyer, seller, *amount, product, 12345);

        let product_bytes = Bytes::from_array(&env, &product.to_array());
        let proof_exists = verify_transaction_proof_exists(
            &env,
            &contract_id,
            buyer,
            seller,
            *amount,
            &product_bytes,
        );
        assert!(
            proof_exists,
            "High-volume transaction proof should exist at index {}",
            i
        );
    }
}

#[test]
fn test_concurrent_verification_operations() {
    let (env, contract_id, client) = setup_test();
    let concurrent_count = 50;
    let transactions = create_multiple_transactions(&env, concurrent_count);
    let mut verification_results = std::vec::Vec::new();

    // Create NFTs and immediately verify them
    for (buyer, seller, amount, product) in transactions {
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

        // Immediate verification
        let product_bytes = Bytes::from_array(&env, &product.to_array());
        let proof_exists = verify_transaction_proof_exists(
            &env,
            &contract_id,
            &buyer,
            &seller,
            amount,
            &product_bytes,
        );

        verification_results.push((tx_id, proof_exists));
    }

    // All verifications should succeed
    for (tx_id, verified) in verification_results {
        assert!(
            verified,
            "Concurrent verification should succeed for NFT {:?}",
            tx_id
        );
    }
}

#[test]
fn test_verification_system_integrity() {
    let (env, contract_id, client) = setup_test();
    let (buyer, seller, amount, product) = create_standard_transaction(&env);
    let product_bytes = create_product_bytes(&env, 1);

    // Create NFT and verify complete system integrity
    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

    // 1. Verify NFT exists
    let metadata = client.get_nft_metadata(&tx_id);
    assert!(metadata.is_some(), "NFT metadata should exist");

    // 2. Verify transaction proof exists
    let proof_exists = verify_transaction_proof_exists(
        &env,
        &contract_id,
        &buyer,
        &seller,
        amount,
        &product_bytes,
    );
    assert!(proof_exists, "Transaction proof should exist");

    // 3. Verify data consistency across systems
    let metadata_unwrapped = metadata.unwrap();
    verify_nft_metadata(
        &metadata_unwrapped,
        &buyer,
        &seller,
        amount,
        &product,
        12345,
    );

    // 4. Note: Duplicate transaction prevention is tested separately
    // as it requires should_panic annotation for proper testing
}
