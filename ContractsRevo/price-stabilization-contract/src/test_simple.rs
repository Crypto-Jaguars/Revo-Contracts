#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, Env,
};

use crate::{
    PriceStabilizationContract,
    PriceStabilizationContractClient,
};

#[test]
fn test_contract_init() {
    let env = Env::default();
    let contract_id = env.register(PriceStabilizationContract, ());
    let client = PriceStabilizationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    env.mock_all_auths();
    client.init(&admin);
    
    // Test passes if no panic occurs
    assert!(true);
}

#[test]
fn test_basic_fund_creation() {
    let env = Env::default();
    let contract_id = env.register(PriceStabilizationContract, ());
    let client = PriceStabilizationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    env.mock_all_auths();
    client.init(&admin);
    
    let fund_name = soroban_sdk::String::from_str(&env, "Test Fund");
    let crop_type = soroban_sdk::String::from_str(&env, "wheat");
    let price_threshold = 10000i128;
    
    let result = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    assert!(result.is_ok());
}

#[test]
fn test_basic_oracle_registration() {
    let env = Env::default();
    let contract_id = env.register(PriceStabilizationContract, ());
    let client = PriceStabilizationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    
    env.mock_all_auths();
    client.init(&admin);
    
    let oracle = Address::generate(&env);
    let crop_type = soroban_sdk::String::from_str(&env, "wheat");
    
    let result = client.try_register_price_oracle(&admin, &oracle, &crop_type);
    assert!(result.is_ok());
}

#[test]
fn test_fund_contribution() {
    let env = Env::default();
    let contract_id = env.register(PriceStabilizationContract, ());
    let client = PriceStabilizationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let contributor = Address::generate(&env);
    
    env.mock_all_auths();
    client.init(&admin);
    
    // Create fund
    let fund_name = soroban_sdk::String::from_str(&env, "Test Fund");
    let crop_type = soroban_sdk::String::from_str(&env, "wheat");
    let price_threshold = 10000i128;
    
    let fund_result = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    assert!(fund_result.is_ok());
    let fund_id = fund_result.unwrap().unwrap();
    
    // Contribute to fund
    let amount = 5000i128;
    let result = client.try_contribute_fund(&contributor, &fund_id, &amount);
    assert!(result.is_ok());
}

#[test]
fn test_comprehensive_workflow() {
    let env = Env::default();
    let contract_id = env.register(PriceStabilizationContract, ());
    let client = PriceStabilizationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let farmer = Address::generate(&env);
    let contributor = Address::generate(&env);
    
    env.mock_all_auths();
    
    // 1. Initialize
    client.init(&admin);
    
    // 2. Create fund
    let fund_name = soroban_sdk::String::from_str(&env, "Wheat Stabilization Fund");
    let crop_type = soroban_sdk::String::from_str(&env, "wheat");
    let price_threshold = 10000i128;
    let fund_id = client.create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    
    // 3. Register oracle
    let _ = client.register_price_oracle(&admin, &oracle, &crop_type);
    
    // 4. Register farmer
    let _ = client.register_farmer(&admin, &farmer);
    let _ = client.register_farmer_crop(&admin, &farmer, &crop_type, &1000i128);
    
    // 5. Add contribution
    let _ = client.contribute_fund(&contributor, &fund_id, &50000i128);
    
    // 6. Update price
    let low_price = 8000i128; // Below threshold
    let timestamp = env.ledger().timestamp();
    let _ = client.update_market_price(&oracle, &crop_type, &low_price, &timestamp);
    
    // 7. Check threshold
    assert!(client.check_price_threshold(&fund_id));
    
    // 8. Verify fund status
    let status_result = client.try_get_fund_status(&fund_id);
    assert!(status_result.is_ok());
}
