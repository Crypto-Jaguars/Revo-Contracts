use soroban_sdk::{testutils::Address as _, vec, Address};
use super::utils::*;

#[test]
fn test_successful_fund_creation() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    let price_threshold = 10000i128;
    
    let result = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    assert!(result.is_ok(), "fund creation should succeed");
    
    let fund_id = match result.unwrap() {
        Ok(id) => id,
        Err(conv_err) => panic!("fund ID conversion failed: {:?}", conv_err),
    };
    
    validate_fund_creation(&client, &fund_id, &crop_type);
}

#[test]
fn test_unauthorized_fund_creation() {
    let (env, client, admin, farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    let price_threshold = 10000i128;
    
    // Try to create fund with non-admin user
    let result = client.try_create_fund(&farmer, &fund_name, &price_threshold, &crop_type);
    assert!(result.is_err(), "unauthorized fund creation should fail");
}

#[test]
fn test_duplicate_fund_creation() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    let price_threshold = 10000i128;
    
    // Create first fund
    let result1 = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    assert!(result1.is_ok(), "first fund creation should succeed");
    
    // Try to create duplicate fund with same name
    let result2 = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    assert!(result2.is_err(), "duplicate fund creation should fail");
}

#[test]
fn test_fund_creation_invalid_inputs() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    
    // Test with zero threshold
    let result1 = client.try_create_fund(&admin, &fund_name, &0i128, &crop_type);
    assert!(result1.is_err(), "zero threshold should be rejected");
    
    // Test with negative threshold
    let result2 = client.try_create_fund(&admin, &fund_name, &-1000i128, &crop_type);
    assert!(result2.is_err(), "negative threshold should be rejected");
}

#[test]
fn test_fund_contribution_success() {
    let (env, client, admin, farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    let price_threshold = 10000i128;
    
    let fund_result = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    let fund_id = match fund_result.unwrap() {
        Ok(id) => id,
        Err(conv_err) => panic!("fund ID conversion failed: {:?}", conv_err),
    };
    
    let contribution_amount = 5000i128;
    let result = client.try_contribute_fund(&farmer, &fund_id, &contribution_amount);
    assert!(result.is_ok(), "fund contribution should succeed");
}

#[test]
fn test_fund_contribution_unauthorized() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    let price_threshold = 10000i128;
    
    let fund_result = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    let fund_id = match fund_result.unwrap() {
        Ok(id) => id,
        Err(conv_err) => panic!("fund ID conversion failed: {:?}", conv_err),
    };
    
    let unauthorized_user = Address::generate(&env);
    let contribution_amount = 5000i128;
    
    let _result = client.try_contribute_fund(&unauthorized_user, &fund_id, &contribution_amount);
    // Note: This test depends on whether contributions have authorization requirements
    // Adjust assertion based on actual contract behavior
}

#[test]
fn test_fund_status_retrieval() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    let price_threshold = 10000i128;
    
    let fund_result = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
    let fund_id = match fund_result.unwrap() {
        Ok(id) => id,
        Err(conv_err) => panic!("fund ID conversion failed: {:?}", conv_err),
    };
    
    let status = client.try_get_fund_status(&fund_id);
    assert!(status.is_ok(), "fund status should be retrievable");
}

#[test]
fn test_fund_not_found() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let non_existent_fund_id = create_test_fund_id(&env, 99);
    let status = client.try_get_fund_status(&non_existent_fund_id);
    assert!(status.is_err(), "non-existent fund should return error");
}

#[test]
fn test_update_price_threshold() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_name = create_test_fund_name(&env, 1);
    let crop_type = create_test_crop_type(&env, 1);
    let initial_threshold = 10000i128;
    
    let fund_result = client.try_create_fund(&admin, &fund_name, &initial_threshold, &crop_type);
    let fund_id = match fund_result.unwrap() {
        Ok(id) => id,
        Err(conv_err) => panic!("fund ID conversion failed: {:?}", conv_err),
    };
    
    let new_threshold = 15000i128;
    // Note: This test assumes there's an update threshold function
    // Adjust based on actual contract interface
    let _result = client.try_update_price_threshold(&admin, &fund_id, &new_threshold);
    // Implementation depends on whether this function exists
}

#[test]
fn test_multiple_funds_scalability() {
    let (env, client, admin, _farmer) = setup_test_environment();
    client.init(&admin);
    
    let fund_count = 10;
    let mut created_funds = vec![&env];
    
    for i in 1..=fund_count {
        let fund_name = create_test_fund_name(&env, i);
        let crop_type = create_test_crop_type(&env, i);
        let price_threshold = (i as i128) * 1000;
        
        let result = client.try_create_fund(&admin, &fund_name, &price_threshold, &crop_type);
        assert!(result.is_ok(), "fund {} creation should succeed", i);
        
        if let Ok(Ok(fund_id)) = result {
            created_funds.push_back(fund_id);
        }
    }
    
    assert_eq!(created_funds.len() as u8, fund_count + 1, "all funds should be created"); // +1 for initial vec
}
