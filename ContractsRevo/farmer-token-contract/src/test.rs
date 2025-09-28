#![cfg(test)]

use crate::{
    AdminError, BurnError, FarmerTokenContract, FarmerTokenContractClient, MintError, TokenError,
};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String, Symbol, Vec};

fn setup_test<'a>() -> (
    Env,
    FarmerTokenContractClient<'a>,
    Address, // Admin
    Address, // Farmer 1
    Address, // Farmer 2
    Address, // Minter
) {
    let env = Env::default();
    env.mock_all_auths();

    // Generate addresses
    let admin = Address::generate(&env);
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);
    let minter = Address::generate(&env);

    // Deploy contract
    let contract_id = env.register(FarmerTokenContract, ());
    let client = FarmerTokenContractClient::new(&env, &contract_id);

    // Initialize contract
    let name = String::from_str(&env, "Farmer Token");
    let symbol = String::from_str(&env, "FRM");
    let decimals = 7u32;

    client.initialize(&admin, &name, &symbol, &decimals);

    (env, client, admin, farmer1, farmer2, minter)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(FarmerTokenContract, ());
    let client = FarmerTokenContractClient::new(&env, &contract_id);

    let name = String::from_str(&env, "Farmer Token");
    let symbol = String::from_str(&env, "FRM");
    let decimals = 7u32;

    // Initialize should succeed
    client.initialize(&admin, &name, &symbol, &decimals);

    // Check metadata
    let metadata = client.token_metadata();
    assert_eq!(metadata.name, name);
    assert_eq!(metadata.symbol, symbol);
    assert_eq!(metadata.decimals, decimals);
    assert_eq!(metadata.total_supply, 0);

    // Check admin
    assert_eq!(client.admin(), admin.clone());

    // Admin should be a minter by default
    assert!(client.is_minter(&admin));

    // Should not be paused
    assert!(!client.is_paused());
}

#[test]
fn test_initialize_already_initialized() {
    let (_, client, admin, _, _, _) = setup_test();
    let name = String::from_str(&client.env, "New Token");
    let symbol = String::from_str(&client.env, "NEW");
    
    let result = client.try_initialize(&admin, &name, &symbol, &7);
    assert_eq!(result, Err(Ok(TokenError::AlreadyInitialized)));
}

#[test]
fn test_mint_tokens() {
    let (_, client, admin, farmer1, _, _) = setup_test();
    
    let amount = 1000_0000000i128; // 1000 tokens with 7 decimals

    // Mint tokens as admin (who is a minter)
    client.mint(&admin, &farmer1, &amount);

    // Check balance
    assert_eq!(client.balance(&farmer1), amount);

    // Check total supply
    assert_eq!(client.total_supply(), amount);
}

#[test]
fn test_mint_unauthorized() {
    let (_, client, _, farmer1, farmer2, _) = setup_test();
    
    let amount = 100_0000000i128;

    // Try to mint without being a minter
    let result = client.try_mint(&farmer1, &farmer2, &amount);
    assert_eq!(result, Err(Ok(MintError::Unauthorized)));
}

#[test]
fn test_mint_invalid_amount() {
    let (_, client, admin, farmer1, _, _) = setup_test();
    
    // Try to mint zero
    let result = client.try_mint(&admin, &farmer1, &0);
    assert_eq!(result, Err(Ok(MintError::InvalidAmount)));

    // Try to mint negative
    let result = client.try_mint(&admin, &farmer1, &-100);
    assert_eq!(result, Err(Ok(MintError::InvalidAmount)));
}

#[test]
fn test_transfer() {
    let (_, client, admin, farmer1, farmer2, _) = setup_test();
    
    let mint_amount = 1000_0000000i128;
    let transfer_amount = 250_0000000i128;

    // Mint tokens to farmer1
    client.mint(&admin, &farmer1, &mint_amount);

    // Transfer from farmer1 to farmer2
    client.transfer(&farmer1, &farmer2, &transfer_amount);

    // Check balances
    assert_eq!(client.balance(&farmer1), mint_amount - transfer_amount);
    assert_eq!(client.balance(&farmer2), transfer_amount);

    // Total supply should remain unchanged
    assert_eq!(client.total_supply(), mint_amount);
}

#[test]
fn test_transfer_insufficient_balance() {
    let (_, client, _, farmer1, farmer2, _) = setup_test();
    
    let result = client.try_transfer(&farmer1, &farmer2, &100);
    assert_eq!(result, Err(Ok(TokenError::InsufficientBalance)));
}

#[test]
fn test_approve_and_transfer_from() {
    let (_, client, admin, farmer1, farmer2, minter) = setup_test();
    
    let mint_amount = 1000_0000000i128;
    let approved_amount = 300_0000000i128;
    let transfer_amount = 200_0000000i128;

    // Mint tokens to farmer1
    client.mint(&admin, &farmer1, &mint_amount);

    // Farmer1 approves minter to spend tokens
    client.approve(&farmer1, &minter, &approved_amount);

    // Check allowance
    assert_eq!(client.allowance(&farmer1, &minter), approved_amount);

    // Minter transfers from farmer1 to farmer2
    client.transfer_from(&minter, &farmer1, &farmer2, &transfer_amount);

    // Check balances
    assert_eq!(client.balance(&farmer1), mint_amount - transfer_amount);
    assert_eq!(client.balance(&farmer2), transfer_amount);

    // Check remaining allowance
    assert_eq!(client.allowance(&farmer1, &minter), approved_amount - transfer_amount);
}

#[test]
fn test_transfer_from_insufficient_allowance() {
    let (_, client, admin, farmer1, farmer2, minter) = setup_test();
    
    let mint_amount = 1000_0000000i128;
    let approved_amount = 100_0000000i128;
    let transfer_amount = 200_0000000i128;

    client.mint(&admin, &farmer1, &mint_amount);
    client.approve(&farmer1, &minter, &approved_amount);

    let result = client.try_transfer_from(&minter, &farmer1, &farmer2, &transfer_amount);
    assert_eq!(result, Err(Ok(TokenError::InsufficientAllowance)));
}

#[test]
fn test_burn_tokens() {
    let (_, client, admin, farmer1, _, _) = setup_test();
    
    let mint_amount = 1000_0000000i128;
    let burn_amount = 250_0000000i128;

    // Mint tokens
    client.mint(&admin, &farmer1, &mint_amount);

    // Burn tokens
    client.burn(&farmer1, &farmer1, &burn_amount);

    // Check balance and total supply
    assert_eq!(client.balance(&farmer1), mint_amount - burn_amount);
    assert_eq!(client.total_supply(), mint_amount - burn_amount);
}

#[test]
fn test_burn_insufficient_balance() {
    let (_, client, admin, farmer1, _, _) = setup_test();
    
    let mint_amount = 100_0000000i128;
    let burn_amount = 200_0000000i128;

    client.mint(&admin, &farmer1, &mint_amount);

    let result = client.try_burn(&farmer1, &farmer1, &burn_amount);
    assert_eq!(result, Err(Ok(BurnError::InsufficientBalance)));
}

#[test]
fn test_add_remove_minter() {
    let (_, client, admin, _, _, minter) = setup_test();
    
    // Add minter
    client.add_minter(&admin, &minter);
    assert!(client.is_minter(&minter));

    // Remove minter
    client.remove_minter(&admin, &minter);
    assert!(!client.is_minter(&minter));
}

#[test]
fn test_add_minter_unauthorized() {
    let (_, client, _, farmer1, _, minter) = setup_test();
    
    let result = client.try_add_minter(&farmer1, &minter);
    assert_eq!(result, Err(Ok(AdminError::Unauthorized)));
}

#[test]
fn test_pause_unpause() {
    let (_, client, admin, farmer1, farmer2, _) = setup_test();
    
    let amount = 100_0000000i128;
    
    // Mint tokens first
    client.mint(&admin, &farmer1, &amount);

    // Pause contract
    client.pause(&admin);
    assert!(client.is_paused());

    // Try to transfer while paused
    let result = client.try_transfer(&farmer1, &farmer2, &amount);
    assert_eq!(result, Err(Ok(TokenError::Paused)));

    // Try to mint while paused
    let result = client.try_mint(&admin, &farmer1, &amount);
    assert_eq!(result, Err(Ok(MintError::Paused)));

    // Unpause contract
    client.unpause(&admin);
    assert!(!client.is_paused());

    // Transfer should work now
    client.transfer(&farmer1, &farmer2, &amount);
}

#[test]
fn test_pause_unauthorized() {
    let (_, client, _, farmer1, _, _) = setup_test();
    
    let result = client.try_pause(&farmer1);
    assert_eq!(result, Err(Ok(AdminError::Unauthorized)));
}

#[test]
fn test_batch_mint() {
    let (env, client, admin, farmer1, farmer2, minter) = setup_test();
    
    // Add minter
    client.add_minter(&admin, &minter);

    let farmer3 = Address::generate(&env);
    let amount1 = 100_0000000i128;
    let amount2 = 200_0000000i128;
    let amount3 = 300_0000000i128;

    // Create recipients vector
    let recipients: Vec<(Address, i128)> = vec![
        &env,
        (farmer1.clone(), amount1),
        (farmer2.clone(), amount2),
        (farmer3.clone(), amount3),
    ];

    // Batch mint
    client.batch_mint(&minter, &recipients);

    // Check balances
    assert_eq!(client.balance(&farmer1), amount1);
    assert_eq!(client.balance(&farmer2), amount2);
    assert_eq!(client.balance(&farmer3), amount3);

    // Check total supply
    assert_eq!(client.total_supply(), amount1 + amount2 + amount3);
}

#[test]
fn test_milestone_mint() {
    let (_, client, admin, farmer1, _, _) = setup_test();
    
    let amount = 500_0000000i128;
    let milestone = Symbol::new(&client.env, "harvest_complete");

    // Mint for milestone
    client.mint_for_milestone(&admin, &farmer1, &milestone, &amount);

    // Check balance
    assert_eq!(client.balance(&farmer1), amount);
}

#[test]
fn test_burn_for_redemption() {
    let (_, client, admin, farmer1, _, _) = setup_test();
    
    let mint_amount = 1000_0000000i128;
    let redeem_amount = 400_0000000i128;
    let redemption_type = Symbol::new(&client.env, "equipment");

    // Mint tokens
    client.mint(&admin, &farmer1, &mint_amount);

    // Burn for redemption
    client.burn_for_redemption(&farmer1, &redeem_amount, &redemption_type);

    // Check balance
    assert_eq!(client.balance(&farmer1), mint_amount - redeem_amount);
}

#[test]
fn test_burn_as_penalty() {
    let (_, client, admin, farmer1, _, _) = setup_test();
    
    let mint_amount = 1000_0000000i128;
    let penalty_amount = 100_0000000i128;
    let reason = Symbol::new(&client.env, "violation");

    // Mint tokens
    client.mint(&admin, &farmer1, &mint_amount);

    // Burn as penalty
    client.burn_as_penalty(&admin, &farmer1, &penalty_amount, &reason);

    // Check balance
    assert_eq!(client.balance(&farmer1), mint_amount - penalty_amount);
}

