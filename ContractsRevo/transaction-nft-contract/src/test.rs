#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        Address, Bytes, BytesN, Env,
    };

    #[test]
    fn test_proof_generation() {
        let env = Env::default();
        let buyer = Address::random(&env);
        let seller = Address::random(&env);
        let amount: u64 = 1000;
        let product = Bytes::from_array(&env, &[1u8; 32]);

        env.ledger().set(LedgerInfo {
            timestamp: 12345,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let tx_id = generate_transaction_proof(env.clone(), buyer.clone(), seller.clone(), amount, product.clone());

        assert_eq!(tx_id.len(), 32, "Transaction ID should be 32 bytes");
        
        assert!(
            transaction_exists(&env, &buyer, &seller, amount, &product),
            "Transaction should exist after proof generation"
        );
    }

    #[test]
    fn test_proof_uniqueness() {
        let env = Env::default();
        let buyer = Address::random(&env);
        let seller = Address::random(&env);
        let amount: u64 = 1000;
        let product = Bytes::from_array(&env, &[1u8; 32]);

        env.ledger().set(LedgerInfo {
            timestamp: 12345,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let proof1 = generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product.clone(),
        );
        
        env.ledger().set(LedgerInfo {
            timestamp: 12346, // Different timestamp
            protocol_version: 1,
            sequence_number: 11,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let proof2 = generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product.clone(),
        );

        assert_ne!(proof1, proof2, "Proofs should be unique even with same input data");
    }

    #[test]
    fn test_nft_minting() {
        let env = Env::default();
        let buyer = Address::random(&env);
        let seller = Address::random(&env);
        let amount: u64 = 1000;
        let product = BytesN::from_array(&env, &[1u8; 32]);

        env.ledger().set(LedgerInfo {
            timestamp: 12345,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        let tx_id = BytesN::from_array(&env, &[2u8; 32]);

        mint_nft(&env, &buyer, tx_id.clone(), &seller, amount, &product);

        let stored_metadata: NFTMetadata = env.storage().instance().get(&tx_id).unwrap();
        assert_eq!(stored_metadata.buyer, buyer, "Buyer mismatch in metadata");
        assert_eq!(stored_metadata.seller, seller, "Seller mismatch in metadata");
        assert_eq!(stored_metadata.amount, amount, "Amount mismatch in metadata");
        assert_eq!(stored_metadata.product, product, "Product mismatch in metadata");
        assert_eq!(stored_metadata.timestamp, 12345, "Timestamp mismatch in metadata");
    }

    #[test]
    #[should_panic(expected = "Invalid ledger timestamp")]
    fn test_mint_with_invalid_timestamp() {
        let env = Env::default();
        let buyer = Address::random(&env);
        let seller = Address::random(&env);
        let amount: u64 = 1000;
        let product = BytesN::from_array(&env, &[1u8; 32]);
        let tx_id = BytesN::from_array(&env, &[2u8; 32]);

        env.ledger().set(LedgerInfo {
            timestamp: 0,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        mint_nft(&env, &buyer, tx_id, &seller, amount, &product);
    }

    #[test]
    fn test_duplicate_transaction_detection() {
        let env = Env::default();
        let buyer = Address::random(&env);
        let seller = Address::random(&env);
        let amount: u64 = 1000;
        let product = Bytes::from_array(&env, &[1u8; 32]);

        env.ledger().set(LedgerInfo {
            timestamp: 12345,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        generate_transaction_proof(
            env.clone(),
            buyer.clone(),
            seller.clone(),
            amount,
            product.clone(),
        );

        assert!(
            transaction_exists(&env, &buyer, &seller, amount, &product),
            "Transaction should be detected as existing"
        );
    }
}
#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    Address, BytesN, Env, IntoVal,
};

#[test]
fn test_metadata_attachment() {
    let env = Env::default();

    env.mock_all_auths();

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    env.ledger().set_timestamp(current_time);

    let contract_id = env.register(TransactionNFTContract, ());
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let product = BytesN::from_array(&env, &[15; 32]);

    // Mint the NFT
    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

    // Verify authorizations
    assert_eq!(
        env.auths(),
        std::vec![
            (
                buyer.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        symbol_short!("mint_nft"),
                        (buyer.clone(), seller.clone(), 100_u64, product.clone()).into_val(&env),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                seller.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        symbol_short!("mint_nft"),
                        (buyer.clone(), seller.clone(), 100_u64, product.clone()).into_val(&env),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
        ]
    );

    // Retrieve the metadata
    let metadata = client.get_nft_metadata(&tx_id).unwrap();

    // Verify metadata structure
    assert!(
        !metadata.product.is_empty(),
        "Product data should not be empty"
    );
    assert!(metadata.amount > 0, "Amount should be positive");

    // Verify purchase details storage
    assert_eq!(metadata.amount, amount);
    assert_eq!(metadata.product, product);

    // Test buyer-seller information
    assert_eq!(metadata.buyer, buyer);
    assert_eq!(metadata.seller, seller);

    // Ensure timestamp is after contract deployment
    assert!(metadata.timestamp >= env.ledger().timestamp());
}

// #[test]
