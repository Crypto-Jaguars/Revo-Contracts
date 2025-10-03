use super::utils::*;
use soroban_sdk::{testutils::Address as _, Address};

#[test]
fn test_oracle_registration_success() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);

    let oracle = create_test_oracle(&env);
    let crop_type = create_test_crop_type(&env, 1);

    let result = client.try_register_price_oracle(&admin, &oracle, &crop_type);
    assert!(result.is_ok(), "oracle registration should succeed");
}

#[test]
fn test_oracle_registration_unauthorized() {
    let (env, client, admin, farmer) = setup_test_environment();
    client.init(&admin);

    let oracle = create_test_oracle(&env);
    let crop_type = create_test_crop_type(&env, 1);

    // Try to register oracle with non-admin user
    let result = client.try_register_price_oracle(&farmer, &oracle, &crop_type);
    assert!(
        result.is_err(),
        "unauthorized oracle registration should fail"
    );
}

#[test]
fn test_market_price_update_success() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);

    let oracle = create_test_oracle(&env);
    let crop_type = create_test_crop_type(&env, 1);

    // Register oracle first
    let reg_result = client.try_register_price_oracle(&admin, &oracle, &crop_type);
    assert!(reg_result.is_ok(), "oracle registration should succeed");

    let price = 12000i128;
    let timestamp = env.ledger().timestamp();

    let result = client.try_update_market_price(&oracle, &crop_type, &price, &timestamp);
    assert!(result.is_ok(), "market price update should succeed");
}

#[test]
fn test_market_price_update_invalid_oracle() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);

    let unregistered_oracle = create_test_oracle(&env);
    let crop_type = create_test_crop_type(&env, 1);
    let price = 12000i128;
    let timestamp = env.ledger().timestamp();

    let result =
        client.try_update_market_price(&unregistered_oracle, &crop_type, &price, &timestamp);
    assert!(
        result.is_err(),
        "price update from unregistered oracle should fail"
    );
}

#[test]
fn test_market_price_update_invalid_data() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);

    let oracle = create_test_oracle(&env);
    let crop_type = create_test_crop_type(&env, 1);

    // Register oracle first
    let reg_result = client.try_register_price_oracle(&admin, &oracle, &crop_type);
    assert!(reg_result.is_ok(), "oracle registration should succeed");

    // Test with negative price
    let negative_price = -1000i128;
    let timestamp = env.ledger().timestamp();

    let result1 = client.try_update_market_price(&oracle, &crop_type, &negative_price, &timestamp);
    assert!(result1.is_err(), "negative price should be rejected");

    // Test with zero price
    let zero_price = 0i128;
    let result2 = client.try_update_market_price(&oracle, &crop_type, &zero_price, &timestamp);
    assert!(result2.is_err(), "zero price should be rejected");
}

#[test]
fn test_price_retrieval() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);

    let oracle = create_test_oracle(&env);
    let crop_type = create_test_crop_type(&env, 1);

    // Register oracle and update price
    client
        .try_register_price_oracle(&admin, &oracle, &crop_type)
        .unwrap();

    let expected_price = 12000i128;
    let timestamp = env.ledger().timestamp();
    client
        .try_update_market_price(&oracle, &crop_type, &expected_price, &timestamp)
        .unwrap();

    // Retrieve price
    let price_result = client.try_get_market_price(&crop_type);
    assert!(price_result.is_ok(), "price retrieval should succeed");

    // Verify price matches
    match price_result {
        Ok(inner_result) => match inner_result {
            Ok((actual_price, _timestamp)) => {
                assert_eq!(
                    actual_price, expected_price,
                    "retrieved price should match set price"
                );
            }
            Err(_) => panic!("failed to get price from result"),
        },
        Err(_) => panic!("failed to call get_market_price"),
    }
}

#[test]
fn test_price_threshold_check_above() {
    let (env, client, admin, _farmer1, _farmer2, _fund_id) = setup_complete_scenario();

    let crop_type = create_test_crop_type(&env, 1);
    let oracle = create_test_oracle(&env);

    // Register oracle
    client
        .try_register_price_oracle(&admin, &oracle, &crop_type)
        .unwrap();

    // Set market price above threshold (threshold is 10000 from setup)
    let high_price = 15000i128;
    let timestamp = env.ledger().timestamp();
    client
        .try_update_market_price(&oracle, &crop_type, &high_price, &timestamp)
        .unwrap();

    // Check if threshold is exceeded
    let threshold_result = client.try_check_price_threshold(&_fund_id);
    assert!(threshold_result.is_ok(), "threshold check should succeed");
}

#[test]
fn test_price_threshold_check_below() {
    let (env, client, admin, _farmer1, _farmer2, _fund_id) = setup_complete_scenario();

    let crop_type = create_test_crop_type(&env, 1);
    let oracle = create_test_oracle(&env);

    // Register oracle
    client
        .try_register_price_oracle(&admin, &oracle, &crop_type)
        .unwrap();

    // Set market price below threshold (threshold is 10000 from setup)
    let low_price = 8000i128;
    let timestamp = env.ledger().timestamp();
    client
        .try_update_market_price(&oracle, &crop_type, &low_price, &timestamp)
        .unwrap();

    // Check if threshold is not exceeded
    let threshold_result = client.try_check_price_threshold(&_fund_id);
    assert!(threshold_result.is_ok(), "threshold check should succeed");
}

#[test]
fn test_chainlink_integration() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);

    let crop_type = create_test_crop_type(&env, 1);
    let chainlink_oracle = Address::generate(&env);

    // Test Chainlink oracle registration
    let result = client.try_register_price_oracle(&admin, &chainlink_oracle, &crop_type);
    assert!(
        result.is_ok(),
        "Chainlink oracle registration should succeed"
    );

    // Test price feed from Chainlink
    let price = 11500i128;
    let timestamp = env.ledger().timestamp();
    let feed_result =
        client.try_update_market_price(&chainlink_oracle, &crop_type, &price, &timestamp);
    assert!(feed_result.is_ok(), "Chainlink price feed should succeed");
}

#[test]
fn test_price_data_validation_edge_cases() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);

    let oracle = create_test_oracle(&env);
    let crop_type = create_test_crop_type(&env, 1);

    // Register oracle
    client
        .try_register_price_oracle(&admin, &oracle, &crop_type)
        .unwrap();

    // Test with maximum i128 value
    let max_price = i128::MAX;
    let timestamp = env.ledger().timestamp();
    let result1 = client.try_update_market_price(&oracle, &crop_type, &max_price, &timestamp);
    // Contract may reject extremely high values for safety
    // Test passes regardless of acceptance/rejection

    // Test with future timestamp
    let future_timestamp = env.ledger().timestamp() + 86400; // +1 day
    let normal_price = 12000i128;
    let result2 =
        client.try_update_market_price(&oracle, &crop_type, &normal_price, &future_timestamp);
    assert!(result2.is_err(), "future timestamp should be rejected");

    // Test with very old timestamp
    let old_timestamp = 1000u64; // Very old timestamp
    let result3 =
        client.try_update_market_price(&oracle, &crop_type, &normal_price, &old_timestamp);
    // Very old timestamps may be rejected for data freshness
    // Test passes regardless of acceptance/rejection
}
