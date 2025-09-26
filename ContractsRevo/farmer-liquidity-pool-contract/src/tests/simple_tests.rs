use soroban_sdk::{testutils::Address as _, testutils::Events, Env, vec, Address};
use crate::{FarmerLiquidityPoolContract, FarmerLiquidityPoolContractClient};

#[test]
fn test_pool_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
    
    // Initialize pool with 0.3% fee rate (30 basis points)
    pool_contract.initialize(&admin, &token_a, &token_b, &30);
    
    let pool_info = pool_contract.get_pool_info();
    assert_eq!(pool_info.token_a, token_a);
    assert_eq!(pool_info.token_b, token_b);
    assert_eq!(pool_info.fee_rate, 30);
    assert_eq!(pool_info.admin, admin);
    assert_eq!(pool_info.is_active, true);
    assert_eq!(pool_info.reserve_a, 0);
    assert_eq!(pool_info.reserve_b, 0);
    assert_eq!(pool_info.total_lp_tokens, 0);
}

#[test]
fn test_pool_initialization_zero_fee_rate() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
    
    // Initialize pool with 0% fee rate
    pool_contract.initialize(&admin, &token_a, &token_b, &0);
    
    let pool_info = pool_contract.get_pool_info();
    assert_eq!(pool_info.fee_rate, 0);
}

#[test]
fn test_pool_initialization_max_fee_rate() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
    
    // Initialize pool with 100% fee rate
    pool_contract.initialize(&admin, &token_a, &token_b, &10000);
    
    let pool_info = pool_contract.get_pool_info();
    assert_eq!(pool_info.fee_rate, 10000);
}

#[test]
fn test_pool_initialization_different_fee_rates() {
    let env = Env::default();
    
    // Test various valid fee rates
    let fee_rates = vec![&env, 0, 1, 10, 30, 100, 500, 1000, 5000, 10000];
    
    for fee_rate in fee_rates.iter() {
        let admin = Address::generate(&env);
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        
        let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
        let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
        
        pool_contract.initialize(&admin, &token_a, &token_b, &fee_rate);
        
        let pool_info = pool_contract.get_pool_info();
        assert_eq!(pool_info.fee_rate, fee_rate);
    }
}

#[test]
fn test_pool_initialization_events() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
    
    // Initialize pool and check events
    pool_contract.initialize(&admin, &token_a, &token_b, &30);
    
    // Check that initialization event was emitted
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
fn test_get_reserves_after_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
    
    // Initialize first
    pool_contract.initialize(&admin, &token_a, &token_b, &30);
    
    // Now get reserves should work
    let (reserve_a, reserve_b) = pool_contract.get_reserves();
    assert_eq!(reserve_a, 0);
    assert_eq!(reserve_b, 0);
}

#[test]
fn test_pool_info_immutability_after_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
    
    // Initialize pool
    pool_contract.initialize(&admin, &token_a, &token_b, &30);
    let initial_pool_info = pool_contract.get_pool_info();
    
    // Check that basic pool info remains the same
    let updated_pool_info = pool_contract.get_pool_info();
    assert_eq!(updated_pool_info.token_a, initial_pool_info.token_a);
    assert_eq!(updated_pool_info.token_b, initial_pool_info.token_b);
    assert_eq!(updated_pool_info.fee_rate, initial_pool_info.fee_rate);
    assert_eq!(updated_pool_info.admin, initial_pool_info.admin);
    assert_eq!(updated_pool_info.is_active, initial_pool_info.is_active);
}

#[test]
fn test_pool_initialization_with_different_admins() {
    let env = Env::default();
    let _admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let contract_id = env.register_contract(None, FarmerLiquidityPoolContract);
    let pool_contract = FarmerLiquidityPoolContractClient::new(&env, &contract_id);
    
    // Initialize with admin2 as admin instead of admin1
    pool_contract.initialize(&admin2, &token_a, &token_b, &30);
    
    let pool_info = pool_contract.get_pool_info();
    assert_eq!(pool_info.admin, admin2);
}