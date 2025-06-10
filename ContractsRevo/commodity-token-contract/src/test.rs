#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

#[test]
fn test_core_storage_operations() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(CommodityTokenContract, ());

    env.as_contract(&contract_id, || {
        // Manually set admin in storage (bypassing require_auth for testing)
        storage::set_admin(&env, &admin);

        // Test inventory management
        let commodity_type = String::from_str(&env, "WHEAT");
        let mut inventory = storage::get_inventory(&env, &commodity_type);
        inventory.total_quantity = 1000;
        inventory.available_quantity = 1000;
        storage::update_inventory(&env, &commodity_type, &inventory).unwrap();

        // Verify inventory was set
        let retrieved_inventory = storage::get_inventory(&env, &commodity_type);
        assert_eq!(retrieved_inventory.total_quantity, 1000);
        assert_eq!(retrieved_inventory.available_quantity, 1000);

        // Test token creation and storage
        let token_id = BytesN::from_array(&env, &[1u8; 32]);
        let token = CommodityBackedToken {
            commodity_type: commodity_type.clone(),
            quantity: 100,
            grade: String::from_str(&env, "A"),
            storage_location: String::from_str(&env, "WAREHOUSE_1"),
            expiration_date: env.ledger().timestamp() + 10000,
            verification_data: BytesN::from_array(&env, &[0u8; 32]),
        };

        storage::store_token(&env, &token_id, &token);
        storage::set_token_owner(&env, &token_id, &admin);

        // Verify token was stored
        let retrieved_token = storage::get_token(&env, &token_id).unwrap();
        assert_eq!(retrieved_token.commodity_type, commodity_type);
        assert_eq!(retrieved_token.quantity, 100);

        // Test token owner retrieval
        let owner = storage::get_token_owner(&env, &token_id).unwrap();
        assert_eq!(owner, admin);
    });
}

#[test]
fn test_contract_initialization_and_empty_inventory() {
    let env = Env::default();
    env.mock_all_auths(); // Mock all auths before contract calls
    let contract_id = env.register(CommodityTokenContract, ());
    let admin = Address::generate(&env);

    // Initialize contract
    env.as_contract(&contract_id, || {
        let result = CommodityTokenContract::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());
    });

    // Test inventory listing (no auth required)
    let commodity_type = String::from_str(&env, "WHEAT");
    let inventory = env.as_contract(&contract_id, || {
        CommodityTokenContract::list_available_inventory(env.clone(), commodity_type.clone())
    });

    // Verify empty inventory initially
    assert_eq!(inventory.total_quantity, 0);
    assert_eq!(inventory.available_quantity, 0);
    assert_eq!(inventory.issued_tokens, 0);
}

#[test]
fn test_commodity_validation_unregistered() {
    let env = Env::default();
    let contract_id = env.register(CommodityTokenContract, ());

    env.as_contract(&contract_id, || {
        let commodity_type = String::from_str(&env, "WHEAT");
        let verification_data = BytesN::from_array(&env, &[0u8; 32]);

        // Test commodity validation (should return false for unregistered commodity)
        let is_valid = CommodityTokenContract::validate_commodity(
            env.clone(),
            commodity_type,
            verification_data,
        );
        assert_eq!(is_valid, false);
    });
}

#[test]
fn test_token_metadata_and_listing_empty() {
    let env = Env::default();
    let contract_id = env.register(CommodityTokenContract, ());

    env.as_contract(&contract_id, || {
        // Test getting non-existent token (should fail)
        let token_id = BytesN::from_array(&env, &[1u8; 32]);
        let result = CommodityTokenContract::get_token_metadata(env.clone(), token_id);
        assert!(result.is_err());

        // Test listing tokens by commodity (should return empty)
        let commodity_type = String::from_str(&env, "WHEAT");
        let tokens = CommodityTokenContract::list_tokens_by_commodity(env.clone(), commodity_type);
        assert_eq!(tokens.len(), 0);
    });
}

#[test]
fn test_admin_and_inventory_management() {
    let env = Env::default();
    let contract_id = env.register(CommodityTokenContract, ());
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Step 1: Set admin manually
        storage::set_admin(&env, &admin);

        // Step 2: Add inventory manually
        let commodity_type = String::from_str(&env, "WHEAT");
        let mut inventory = storage::get_inventory(&env, &commodity_type);
        inventory.total_quantity = 1000;
        inventory.available_quantity = 1000;
        storage::update_inventory(&env, &commodity_type, &inventory).unwrap();

        // Step 3: Check inventory
        let retrieved_inventory =
            CommodityTokenContract::list_available_inventory(env.clone(), commodity_type);
        assert_eq!(retrieved_inventory.total_quantity, 1000);
        assert_eq!(retrieved_inventory.available_quantity, 1000);
        assert_eq!(retrieved_inventory.issued_tokens, 0);
    });
}
