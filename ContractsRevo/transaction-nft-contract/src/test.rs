use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};
use crate::{TransactionNFTContract, TransactionNFTContractClient};

#[test]
fn test_successful_nft_minting() {
    let env = Env::default();
    let contract_id = env.register(None, TransactionNFTContract);
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    // Create test addresses
    let buyer = Address::from_str(&env, "buyer_address");
    let seller = Address::from_str(&env, "seller_address");
    
    // Mock product data
    let product_data = BytesN::from_array(&env, &[1u8; 32]);
    
    // Set transaction amount
    let amount = 1000u64;

    // Authorize both parties
    env.mock_all_auths();

    // Mint NFT
    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product_data);

    // Verify NFT metadata
    let metadata = client.get_nft_metadata(&tx_id);
    assert!(metadata.is_some());
    
    let metadata = metadata.unwrap();
    assert_eq!(metadata.buyer, buyer);
    assert_eq!(metadata.seller, seller);
    assert_eq!(metadata.amount, amount);
    assert_eq!(metadata.product, product_data);
    assert!(metadata.timestamp > 0);
}

#[test]
#[should_panic(expected = "Duplicate transaction detected")]
fn test_duplicate_transaction_prevention() {
    let env = Env::default();
    let contract_id = env.register(None, TransactionNFTContract);
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    let buyer = Address::from_str(&env, "buyer_address");
    let seller = Address::from_str(&env, "seller_address");
    let product_data = BytesN::from_array(&env, &[1u8; 32]);
    let amount = 1000u64;

    env.mock_all_auths();

    // First mint should succeed
    client.mint_nft(&buyer, &seller, &amount, &product_data);

    // Second mint with same parameters should fail
    client.mint_nft(&buyer, &seller, &amount, &product_data);
}

#[test]
#[should_panic(expected = "Buyer and seller cannot be the same address")]
fn test_same_buyer_seller_prevention() {
    let env = Env::default();
    let contract_id = env.register(None, TransactionNFTContract);
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    let address = Address::from_str(&env, "same_address");
    let product_data = BytesN::from_array(&env, &[1u8; 32]);
    let amount = 1000u64;

    env.mock_all_auths();

    // Should panic when buyer and seller are the same
    client.mint_nft(&address, &address, &amount, &product_data);
}

#[test]
#[should_panic(expected = "Amount must be greater than zero")]
fn test_zero_amount_prevention() {
    let env = Env::default();
    let contract_id = env.register(None, TransactionNFTContract);
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    let buyer = Address::from_str(&env, "buyer_address");
    let seller = Address::from_str(&env, "seller_address");
    let product_data = BytesN::from_array(&env, &[1u8; 32]);
    
    env.mock_all_auths();

    // Should panic with zero amount
    client.mint_nft(&buyer, &seller, &0u64, &product_data);
}

#[test]
fn test_unique_transaction_ids() {
    let env = Env::default();
    let contract_id = env.register(None, TransactionNFTContract);
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    let buyer = Address::from_str(&env, "buyer_address");
    let seller = Address::from_str(&env, "seller_address");
    let product_data = BytesN::from_array(&env, &[1u8; 32]);
    
    env.mock_all_auths();

    // Create two transactions with different amounts
    let tx_id1 = client.mint_nft(&buyer, &seller, &1000u64, &product_data);
    let tx_id2 = client.mint_nft(&buyer, &seller, &2000u64, &product_data);

    // Verify transaction IDs are different
    assert_ne!(tx_id1, tx_id2);
}

#[test]
fn test_timestamp_tracking() {
    let env = Env::default();
    let contract_id = env.register(None, TransactionNFTContract);
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    let buyer = Address::from_str(&env, "buyer_address");
    let seller = Address::from_str(&env, "seller_address");
    let product_data = BytesN::from_array(&env, &[1u8; 32]);
    let amount = 1000u64;

    // Set a specific timestamp
    env.ledger().set(Ledger {
        timestamp: 12345,
        ..Default::default()
    });
    
    env.mock_all_auths();

    let tx_id = client.mint_nft(&buyer, &seller, &amount, &product_data);
    let metadata = client.get_nft_metadata(&tx_id).unwrap();

    assert_eq!(metadata.timestamp, 12345);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_unauthorized_minting() {
    let env = Env::default();
    let contract_id = env.register(None, TransactionNFTContract);
    let client = TransactionNFTContractClient::new(&env, &contract_id);

    let buyer = Address::from_str(&env, "GABCD1234BUYERADDRESS").unwrap();
    let seller = Address::from_str(&env, "GXYZ5678SELLERADDRESS").unwrap();
    let product_data = BytesN::from_array(&env, &[1u8; 32]);
    let amount = 1000u64;

    // Don't mock authorizations - should panic
    client.mint_nft(&buyer, &seller, &amount, &product_data);
}