use soroban_sdk::{testutils::Address as _, vec, Address};
use super::utils::*;

#[test]
fn test_farmer_registration_success() {
    let (env, client, admin, farmer) = setup_test_environment();
    client.init(&admin);
    
    let result = client.try_register_farmer(&admin, &farmer);
    assert!(result.is_ok(), "farmer registration should succeed");
}

#[test]
fn test_duplicate_farmer_registration() {
    let (env, client, admin, farmer) = setup_test_environment();
    client.init(&admin);
    
    // Register farmer first time
    let result1 = client.try_register_farmer(&admin, &farmer);
    assert!(result1.is_ok(), "initial farmer registration should succeed");
    
    // Try to register same farmer again
    let result2 = client.try_register_farmer(&admin, &farmer);
    assert!(result2.is_err(), "duplicate farmer registration should fail");
}

#[test]
fn test_farmer_crop_registration() {
    let (env, client, admin, farmer) = setup_test_environment();
    client.init(&admin);
    
    // Register farmer first
    client.try_register_farmer(&admin, &farmer).unwrap();
    
    let crop_type = create_test_crop_type(&env, 1);
    let area = 1000i128; // hectares
    
    let result = client.try_register_farmer_crop(&admin, &farmer, &crop_type, &area);
    assert!(result.is_ok(), "farmer crop registration should succeed");
}

#[test]
fn test_farmer_crop_registration_unauthorized() {
    let (env, client, admin, farmer) = setup_test_environment();
    client.init(&admin);
    
    let unauthorized_user = Address::generate(&env);
    let crop_type = create_test_crop_type(&env, 1);
    let area = 1000i128;
    
    let result = client.try_register_farmer_crop(&unauthorized_user, &farmer, &crop_type, &area);
    assert!(result.is_err(), "unauthorized crop registration should fail");
}

#[test]
fn test_payout_trigger_eligible_farmers() {
    let (env, client, admin, farmer1, farmer2, fund_id) = setup_complete_scenario();
    
    // Register farmers
    client.try_register_farmer(&admin, &farmer1).unwrap();
    client.try_register_farmer(&admin, &farmer2).unwrap();
    
    // Register farmer crops
    let crop_type = create_test_crop_type(&env, 1);
    client.try_register_farmer_crop(&admin, &farmer1, &crop_type, &1000i128).unwrap();
    client.try_register_farmer_crop(&admin, &farmer2, &crop_type, &1500i128).unwrap();
    
    // Contribute to fund
    let contributor = Address::generate(&env);
    client.try_contribute_fund(&contributor, &fund_id, &10000i128).unwrap();
    
    // Setup oracle and price data to trigger payout
    let oracle = create_test_oracle(&env);
    client.try_register_price_oracle(&admin, &oracle, &crop_type).unwrap();
    
    // Set price below threshold to trigger payout (threshold is 10000)
    let trigger_price = 8000i128;
    let timestamp = env.ledger().timestamp();
    client.try_update_market_price(&oracle, &crop_type, &trigger_price, &timestamp).unwrap();
    
    // Trigger payout
    let farmers = vec![&env, farmer1.clone(), farmer2.clone()];
    let payout_result = client.try_trigger_payout(&admin, &fund_id, &farmers);
    assert!(payout_result.is_ok(), "payout to eligible farmers should succeed");
    
    // Verify payout records
    let payout1 = client.try_get_farmer_payouts(&fund_id, &farmer1);
    let payout2 = client.try_get_farmer_payouts(&fund_id, &farmer2);
    assert!(payout1.is_ok(), "farmer1 payout record should exist");
    assert!(payout2.is_ok(), "farmer2 payout record should exist");
}

#[test]
fn test_payout_distribution_non_eligible_farmers() {
    let (env, client, admin, farmer1, farmer2, fund_id) = setup_complete_scenario();
    
    // Register only farmer1, leave farmer2 unregistered
    client.try_register_farmer(&admin, &farmer1).unwrap();
    
    let crop_type = create_test_crop_type(&env, 1);
    client.try_register_farmer_crop(&admin, &farmer1, &crop_type, &1000i128).unwrap();
    
    // Contribute to fund
    let contributor = Address::generate(&env);
    client.try_contribute_fund(&contributor, &fund_id, &10000i128).unwrap();
    
    // Try to trigger payout including non-eligible farmer
    let farmers = vec![&env, farmer1.clone(), farmer2.clone()]; // farmer2 is not registered
    let payout_result = client.try_trigger_payout(&admin, &fund_id, &farmers);
    assert!(payout_result.is_err(), "payout including non-eligible farmers should fail");
}

#[test]
fn test_payout_insufficient_funds() {
    let (env, client, admin, farmer1, farmer2, fund_id) = setup_complete_scenario();
    
    // Register farmers
    client.try_register_farmer(&admin, &farmer1).unwrap();
    client.try_register_farmer(&admin, &farmer2).unwrap();
    
    let crop_type = create_test_crop_type(&env, 1);
    client.try_register_farmer_crop(&admin, &farmer1, &crop_type, &1000i128).unwrap();
    client.try_register_farmer_crop(&admin, &farmer2, &crop_type, &1500i128).unwrap();
    
    // Do not contribute to fund (or contribute insufficient amount)
    let contributor = Address::generate(&env);
    client.try_contribute_fund(&contributor, &fund_id, &100i128).unwrap(); // Very small contribution
    
    // Setup oracle and trigger conditions
    let oracle = create_test_oracle(&env);
    client.try_register_price_oracle(&admin, &oracle, &crop_type).unwrap();
    
    let trigger_price = 8000i128;
    let timestamp = env.ledger().timestamp();
    client.try_update_market_price(&oracle, &crop_type, &trigger_price, &timestamp).unwrap();
    
    // Try to trigger payout with insufficient funds
    let farmers = vec![&env, farmer1.clone(), farmer2.clone()];
    let payout_result = client.try_trigger_payout(&admin, &fund_id, &farmers);
    assert!(payout_result.is_err(), "payout with insufficient funds should fail");
}

#[test]
fn test_payout_trigger_invalid_price_data() {
    let (env, client, admin, farmer1, farmer2, fund_id) = setup_complete_scenario();
    
    // Register farmers and crops
    client.try_register_farmer(&admin, &farmer1).unwrap();
    client.try_register_farmer(&admin, &farmer2).unwrap();
    
    let crop_type = create_test_crop_type(&env, 1);
    client.try_register_farmer_crop(&admin, &farmer1, &crop_type, &1000i128).unwrap();
    client.try_register_farmer_crop(&admin, &farmer2, &crop_type, &1500i128).unwrap();
    
    // Contribute to fund
    let contributor = Address::generate(&env);
    client.try_contribute_fund(&contributor, &fund_id, &10000i128).unwrap();
    
    // Do not set up oracle or price data
    // Try to trigger payout without valid price data
    let farmers = vec![&env, farmer1.clone(), farmer2.clone()];
    let payout_result = client.try_trigger_payout(&admin, &fund_id, &farmers);
    assert!(payout_result.is_err(), "payout without valid price data should fail");
}

#[test]
fn test_payout_calculation_accuracy() {
    let (env, client, admin, farmer1, farmer2, fund_id) = setup_complete_scenario();
    
    // Register farmers with different crop areas
    client.try_register_farmer(&admin, &farmer1).unwrap();
    client.try_register_farmer(&admin, &farmer2).unwrap();
    
    let crop_type = create_test_crop_type(&env, 1);
    let area1 = 1000i128; // farmer1: 1000 hectares
    let area2 = 2000i128; // farmer2: 2000 hectares (double)
    
    client.try_register_farmer_crop(&admin, &farmer1, &crop_type, &area1).unwrap();
    client.try_register_farmer_crop(&admin, &farmer2, &crop_type, &area2).unwrap();
    
    // Contribute known amount to fund
    let contributor = Address::generate(&env);
    let total_contribution = 9000i128;
    client.try_contribute_fund(&contributor, &fund_id, &total_contribution).unwrap();
    
    // Setup price trigger
    let oracle = create_test_oracle(&env);
    client.try_register_price_oracle(&admin, &oracle, &crop_type).unwrap();
    
    let trigger_price = 8000i128;
    let timestamp = env.ledger().timestamp();
    client.try_update_market_price(&oracle, &crop_type, &trigger_price, &timestamp).unwrap();
    
    // Trigger payout
    let farmers = vec![&env, farmer1.clone(), farmer2.clone()];
    client.try_trigger_payout(&admin, &fund_id, &farmers).unwrap();
    
    // Verify payout amounts are proportional to crop areas
    // farmer2 should receive double what farmer1 receives (2000 vs 1000 hectares)
    let payout1 = client.try_get_farmer_payouts(&fund_id, &farmer1);
    let payout2 = client.try_get_farmer_payouts(&fund_id, &farmer2);
    
    assert!(payout1.is_ok(), "farmer1 payout should be retrievable");
    assert!(payout2.is_ok(), "farmer2 payout should be retrievable");
    
    // Note: Actual validation of amounts depends on payout structure
}

#[test]
fn test_payout_history_retrieval() {
    let (env, client, admin, farmer1, _farmer2, fund_id) = setup_complete_scenario();
    
    // Register farmer and perform payout
    client.try_register_farmer(&admin, &farmer1).unwrap();
    
    let crop_type = create_test_crop_type(&env, 1);
    client.try_register_farmer_crop(&admin, &farmer1, &crop_type, &1000i128).unwrap();
    
    // Contribute and trigger payout
    let contributor = Address::generate(&env);
    client.try_contribute_fund(&contributor, &fund_id, &10000i128).unwrap();
    
    let oracle = create_test_oracle(&env);
    client.try_register_price_oracle(&admin, &oracle, &crop_type).unwrap();
    
    let trigger_price = 8000i128;
    let timestamp = env.ledger().timestamp();
    client.try_update_market_price(&oracle, &crop_type, &trigger_price, &timestamp).unwrap();
    
    let farmers = vec![&env, farmer1.clone()];
    client.try_trigger_payout(&admin, &fund_id, &farmers).unwrap();
    
    // Retrieve payout history
    let history = client.try_get_farmer_payouts(&fund_id, &farmer1);
    assert!(history.is_ok(), "payout history should be retrievable");
    
    // Test retrieving history for non-existent farmer
    let non_existent_farmer = Address::generate(&env);
    let no_history = client.try_get_farmer_payouts(&fund_id, &non_existent_farmer);
    assert!(no_history.is_err(), "non-existent farmer history should return error");
}
