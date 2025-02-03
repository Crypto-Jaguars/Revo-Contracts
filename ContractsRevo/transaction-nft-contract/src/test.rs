#[cfg(test)]
mod tests {
    use crate::mint::{mint_nft, NFTMetadata};
    use crate::proof::{generate_transaction_proof, transaction_exists};
    use crate::TransactionNFTContract;
    use crate::TransactionNFTContractClient;
    use soroban_sdk::{
        testutils::{Address as _, Events as _, Ledger, LedgerInfo}, Address, Bytes, BytesN, Env, FromVal,
    };

    fn create_ledger_info(timestamp: u64) -> LedgerInfo {
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

    fn create_test_env() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited();
        env.ledger().set(create_ledger_info(12345));
        let contract_id = env.register(TransactionNFTContract, ());
        (env, contract_id)
    }

    #[test]
    fn test_proof_generation() {
        let (env, contract_id) = create_test_env();
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount: u64 = 1000;
        let product = Bytes::from_array(&env, &[1u8; 32]);

        let tx_id = env.as_contract(&contract_id, || {
            generate_transaction_proof(
                env.clone(),
                buyer.clone(),
                seller.clone(),
                amount,
                product.clone(),
            )
        });

        assert_eq!(tx_id.len(), 32, "Transaction ID should be 32 bytes");

        assert!(
            env.as_contract(&contract_id, || {
                transaction_exists(&env, &buyer, &seller, amount, &product)
            }),
            "Transaction should exist after proof generation"
        );
    }

    #[test]
    fn test_proof_uniqueness() {
        let (env, contract_id) = create_test_env();
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount: u64 = 1000;
        let product = Bytes::from_array(&env, &[1u8; 32]);

        let proof1 = env.as_contract(&contract_id, || {
            generate_transaction_proof(
                env.clone(),
                buyer.clone(),
                seller.clone(),
                amount,
                product.clone(),
            )
        });

        env.ledger().set(create_ledger_info(12346));

        let proof2 = env.as_contract(&contract_id, || {
            generate_transaction_proof(
                env.clone(),
                buyer.clone(),
                seller.clone(),
                amount,
                product.clone(),
            )
        });

        assert_ne!(
            proof1, proof2,
            "Proofs should be unique even with same input data"
        );
    }

    #[test]
    fn test_nft_minting() {
        let (env, contract_id) = create_test_env();
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount: u64 = 1000;
        let product = BytesN::from_array(&env, &[1u8; 32]);
        let tx_id = BytesN::from_array(&env, &[2u8; 32]);

        env.as_contract(&contract_id, || {
            mint_nft(&env, &buyer, tx_id.clone(), &seller, amount, &product)
        });

        let stored_metadata = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, NFTMetadata>(&tx_id)
                .unwrap()
        });

        assert_eq!(stored_metadata.buyer, buyer, "Buyer mismatch in metadata");
        assert_eq!(
            stored_metadata.seller, seller,
            "Seller mismatch in metadata"
        );
        assert_eq!(
            stored_metadata.amount, amount,
            "Amount mismatch in metadata"
        );
        assert_eq!(
            stored_metadata.product, product,
            "Product mismatch in metadata"
        );
        assert_eq!(
            stored_metadata.timestamp, 12345,
            "Timestamp mismatch in metadata"
        );
    }

    #[test]
    #[should_panic(expected = "Invalid ledger timestamp")]
    fn test_mint_with_invalid_timestamp() {
        let env = Env::default();
        env.mock_all_auths();
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount: u64 = 1000;
        let product = BytesN::from_array(&env, &[1u8; 32]);
        let tx_id = BytesN::from_array(&env, &[2u8; 32]);

        let mut invalid_info = create_ledger_info(0);
        invalid_info.timestamp = 0;
        env.ledger().set(invalid_info);

        let contract_id = env.register(TransactionNFTContract, ());
        env.as_contract(&contract_id, || {
            mint_nft(&env, &buyer, tx_id, &seller, amount, &product)
        });
    }

    #[test]
    fn test_duplicate_transaction_detection() {
        let (env, contract_id) = create_test_env();
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount: u64 = 1000;
        let product = Bytes::from_array(&env, &[1u8; 32]);

        env.as_contract(&contract_id, || {
            generate_transaction_proof(
                env.clone(),
                buyer.clone(),
                seller.clone(),
                amount,
                product.clone(),
            )
        });

        assert!(
            env.as_contract(&contract_id, || {
                transaction_exists(&env, &buyer, &seller, amount, &product)
            }),
            "Transaction should be detected as existing"
        );
    }

    #[test]
    fn test_metadata_attachment() {
        let (env, contract_id) = create_test_env();
        let client = TransactionNFTContractClient::new(&env, &contract_id);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount = 100;
        let product = BytesN::from_array(&env, &[15; 32]);

        // Mint the NFT
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

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

    #[test]
    fn test_unique_nft_generation() {
        let (env, contract_id) = create_test_env();
        let client = TransactionNFTContractClient::new(&env, &contract_id);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount = 100;
        let product = BytesN::from_array(&env, &[15; 32]);

        // Mint the first NFT
        let tx_id1 = client.mint_nft(&buyer, &seller, &amount, &product);

        // Mint the second NFT with different product data
        let product2 = BytesN::from_array(&env, &[16; 32]);
        let tx_id2 = client.mint_nft(&buyer, &seller, &amount, &product2);

        // Verify that the two transaction IDs are unique
        assert_ne!(tx_id1, tx_id2, "Transaction IDs should be unique for different products");
    }

    #[test]
    fn test_purchase_event_handling() {
        let (env, contract_id) = create_test_env();
        let client = TransactionNFTContractClient::new(&env, &contract_id);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount = 100;
        let product = BytesN::from_array(&env, &[15; 32]);

        // Mint the NFT
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

        // Check for the purchase event
        let events = env.events().all();
        assert!(
            events.iter().any(|(_, _, value)| {
                let value_as_bytes = BytesN::<32>::from_val(&env, &value);
                value_as_bytes == tx_id
            }),
            "NFT minting event should be emitted"
        );
    }

    #[test]
    #[should_panic(expected = "Buyer and seller cannot be the same address")]
    fn test_minting_conditions_same_address() {
        let (env, contract_id) = create_test_env();
        let client = TransactionNFTContractClient::new(&env, &contract_id);

        let buyer = Address::generate(&env);
        let amount = 100;
        let product = BytesN::from_array(&env, &[15; 32]);

        // Test minting with buyer and seller being the same address
        client.mint_nft(&buyer, &buyer, &amount, &product);
    }

    #[test]
    #[should_panic(expected = "Amount must be greater than zero")]
    fn test_minting_conditions_zero_amount() {
        let (env, contract_id) = create_test_env();
        let client = TransactionNFTContractClient::new(&env, &contract_id);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let product = BytesN::from_array(&env, &[15; 32]);

        // Test minting with zero amount
        client.mint_nft(&buyer, &seller, &0, &product);
    }

    #[test]
    fn test_timing_and_sequence() {
        let (env, contract_id) = create_test_env();
        let client = TransactionNFTContractClient::new(&env, &contract_id);

        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
        let amount = 100;
        let product = BytesN::from_array(&env, &[15; 32]);

        // Mint the NFT
        let tx_id = client.mint_nft(&buyer, &seller, &amount, &product);

        // Retrieve the metadata
        let metadata = client.get_nft_metadata(&tx_id).unwrap();

        // Ensure timestamp is correct
        assert_eq!(metadata.timestamp, 12345, "Timestamp should match the ledger info");
    }

}
