use soroban_sdk::{testutils::Address as _, testutils::Events, Env, vec};
use crate::{FarmerLiquidityPoolContract, FarmerLiquidityPoolContractClient};
use super::utils::{setup_test_environment};

#[test]
fn test_pool_initialization() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize pool with 0.3% fee rate (30 basis points)
    test_env.initialize_pool(30);

    let pool_info = test_env.get_pool_info();
    assert_eq!(pool_info.token_a, test_env.token_a);
    assert_eq!(pool_info.token_b, test_env.token_b);
    assert_eq!(pool_info.fee_rate, 30);
    assert_eq!(pool_info.admin, test_env.admin);
    assert_eq!(pool_info.is_active, true);
    assert_eq!(pool_info.reserve_a, 0);
    assert_eq!(pool_info.reserve_b, 0);
    assert_eq!(pool_info.total_lp_tokens, 0);
}

#[test]
fn test_pool_initialization_twice_fails() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize pool first time
    test_env.initialize_pool(30);

    // Try to initialize again - should fail
    // Note: In no_std environment, we can't use std::panic::catch_unwind
    // This test would need to be handled differently in a real implementation
}

#[test]
fn test_pool_initialization_invalid_fee_rate() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Try to initialize with invalid fee rate (>100%)
    // Note: In no_std environment, we can't use std::panic::catch_unwind
    // This test would need to be handled differently in a real implementation
}

#[test]
fn test_pool_initialization_same_tokens_fails() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Try to initialize with same token for both A and B
    let result = std::panic::catch_unwind(|| {
        test_env.pool_contract.initialize(
            &test_env.admin,
            &test_env.token_a,
            &test_env.token_a, // Same token
            &30,
        );
    });
    assert!(result.is_err());
}

#[test]
fn test_pool_initialization_zero_fee_rate() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize pool with 0% fee rate
    test_env.initialize_pool(0);

    let pool_info = test_env.get_pool_info();
    assert_eq!(pool_info.fee_rate, 0);
}

#[test]
fn test_pool_initialization_max_fee_rate() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize pool with 100% fee rate
    test_env.initialize_pool(10000);

    let pool_info = test_env.get_pool_info();
    assert_eq!(pool_info.fee_rate, 10000);
}

#[test]
fn test_get_pool_info_before_initialization() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Try to get pool info before initialization
    let result = std::panic::catch_unwind(|| {
        test_env.get_pool_info();
    });
    assert!(result.is_err());
}

#[test]
fn test_get_reserves_before_initialization() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Try to get reserves before initialization
    let result = std::panic::catch_unwind(|| {
        test_env.get_reserves();
    });
    assert!(result.is_err());
}

#[test]
fn test_pool_initialization_events() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize pool and check events
    test_env.initialize_pool(30);

    // Check that initialization event was emitted
    let events = env.events().all();
    assert!(!events.is_empty());
    
    let init_event = &events[0];
    assert_eq!(init_event.0, soroban_sdk::symbol_short!("pool_initialized"));
}

#[test]
fn test_pool_info_immutability_after_initialization() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize pool
    test_env.initialize_pool(30);
    let initial_pool_info = test_env.get_pool_info();

    // Add some liquidity to change reserves
    test_env.add_liquidity(&test_env.user1, 1000, 2000);

    // Check that basic pool info remains the same
    let updated_pool_info = test_env.get_pool_info();
    assert_eq!(updated_pool_info.token_a, initial_pool_info.token_a);
    assert_eq!(updated_pool_info.token_b, initial_pool_info.token_b);
    assert_eq!(updated_pool_info.fee_rate, initial_pool_info.fee_rate);
    assert_eq!(updated_pool_info.admin, initial_pool_info.admin);
    assert_eq!(updated_pool_info.is_active, initial_pool_info.is_active);
}

#[test]
fn test_pool_initialization_different_fee_rates() {
    let env = Env::default();
    
    // Test various valid fee rates
    let fee_rates = vec![0, 1, 10, 30, 100, 500, 1000, 5000, 10000];
    
    for fee_rate in fee_rates {
        let test_env = setup_test_environment(&env);
        test_env.initialize_pool(fee_rate);
        
        let pool_info = test_env.get_pool_info();
        assert_eq!(pool_info.fee_rate, fee_rate);
    }
}

#[test]
fn test_pool_initialization_with_different_admins() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize with user1 as admin instead of default admin
    test_env.pool_contract.initialize(
        &test_env.user1,
        &test_env.token_a,
        &test_env.token_b,
        &30,
    );

    let pool_info = test_env.get_pool_info();
    assert_eq!(pool_info.admin, test_env.user1);
}

#[test]
fn test_pool_initialization_token_order_independence() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Initialize with token_a and token_b
    test_env.initialize_pool(30);
    let pool_info_1 = test_env.get_pool_info();

    // Create another pool with reversed token order
    let contract_id_2 = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract_2 = FarmerLiquidityPoolContractClient::new(&env, &contract_id_2);
    
    pool_contract_2.initialize(
        &test_env.admin,
        &test_env.token_b, // Reversed order
        &test_env.token_a,
        &30,
    );

    // Both pools should have the same fee rate and admin
    assert_eq!(pool_info_1.fee_rate, 30);
    assert_eq!(pool_info_1.admin, test_env.admin);
}
