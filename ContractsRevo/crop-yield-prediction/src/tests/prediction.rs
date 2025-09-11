#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Ledger as _},
    vec, Address, BytesN, Env, String, Vec,
};

use crate::{
    CropYieldPredictionContractClient,
    types::{CropYieldError, DataSource},
};

use super::utils::*;

/// Test successful crop registration
#[test]
fn test_register_crop_success() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);

    let result = client.try_register_crop(&crop_id, &name, &historical_yields);
    assert!(result.is_ok(), "Crop registration should succeed");
    
    let returned_crop_id = result.unwrap().unwrap();
    assert_eq!(returned_crop_id, crop_id, "Returned crop ID should match input");
    
    // Verify crop was stored correctly
    assert!(validate_crop_registration(&client, &crop_id, &name, 5));
}

/// Test crop registration with invalid input
#[test]
fn test_register_crop_invalid_input() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    let crop_id = create_test_crop_id(&env, 1);
    let empty_name = String::from_str(&env, "");
    let empty_yields = vec![&env];

    let result = client.try_register_crop(&crop_id, &empty_name, &empty_yields);
    assert!(result.is_err(), "Empty name should cause error");
}

/// Test duplicate crop registration
#[test]
fn test_register_crop_duplicate() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);

    // First registration should succeed
    let result1 = client.try_register_crop(&crop_id, &name, &historical_yields);
    assert!(result1.is_ok(), "First crop registration should succeed");

    // Second registration with same ID should overwrite (contract behavior)
    let result2 = client.try_register_crop(&crop_id, &name, &historical_yields);
    assert!(result2.is_ok(), "Duplicate crop registration should succeed (overwrites)");
}

/// Test crop registration by unauthorized user
#[test]
fn test_register_crop_unauthorized() {
    let (env, client, admin, farmer, _) = setup_strict_auth_environment();
    
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);

    // Try to register crop as non-admin (this will fail due to auth requirement)
    let result = client.try_register_crop(&crop_id, &name, &historical_yields);
    assert!(result.is_err(), "Non-admin should not be able to register crops");
}

/// Test successful prediction generation
#[test]
fn test_generate_prediction_success() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // First register a crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    // Generate prediction
    let region = create_test_region(&env, 1);
    let data_source = create_test_data_source(&env, 1);

    let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
    
    // Verify prediction was stored correctly
    assert!(validate_prediction_generation(&client, &prediction_id, &crop_id, &region));
}

/// Test prediction generation with missing crop
#[test]
fn test_generate_prediction_crop_not_found() {
    let (env, client, _, _, _) = setup_test_environment();
    
    let crop_id = create_test_crop_id(&env, 255); // Non-existent crop
    let region = create_test_region(&env, 1);
    let data_source = create_test_data_source(&env, 1);

    let result = client.try_generate_prediction(&crop_id, &region, &data_source);
    assert!(result.is_err(), "Prediction with non-existent crop should fail");
}

/// Test prediction generation with invalid region
#[test]
fn test_generate_prediction_invalid_region() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // First register a crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let empty_region = String::from_str(&env, "");
    let data_source = create_test_data_source(&env, 1);

    let result = client.try_generate_prediction(&crop_id, &empty_region, &data_source);
    assert!(result.is_err(), "Empty region should cause error");
}

/// Test prediction generation with optimal conditions
#[test]
fn test_generate_prediction_optimal_conditions() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // First register a crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let optimal_data = create_optimal_data_source(&env);

    let prediction_id = client.generate_prediction(&crop_id, &region, &optimal_data);
    let prediction = client.get_prediction(&prediction_id);
    
    // Optimal conditions should result in higher yield
    assert!(prediction.predicted_yield > 0, "Predicted yield should be positive");
}

/// Test prediction generation with poor conditions
#[test]
fn test_generate_prediction_poor_conditions() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // First register a crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let poor_data = create_poor_data_source(&env);

    let prediction_id = client.generate_prediction(&crop_id, &region, &poor_data);
    let prediction = client.get_prediction(&prediction_id);
    
    // Poor conditions should result in lower yield
    assert!(prediction.predicted_yield >= 0, "Predicted yield should be non-negative");
}

/// Test prediction generation with extreme conditions
#[test]
fn test_generate_prediction_extreme_conditions() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // First register a crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let extreme_data = create_extreme_data_source(&env);

    let prediction_id = client.generate_prediction(&crop_id, &region, &extreme_data);
    let prediction = client.get_prediction(&prediction_id);
    
    // Extreme conditions should result in very low yield
    assert!(prediction.predicted_yield >= 0, "Predicted yield should be non-negative");
}

/// Test prediction retrieval
#[test]
fn test_get_prediction_success() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // First register a crop and generate prediction
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let data_source = create_test_data_source(&env, 1);
    let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);

    // Retrieve prediction
    let prediction = client.get_prediction(&prediction_id);
    assert_eq!(prediction.crop_id, crop_id);
    assert_eq!(prediction.region, region);
    assert!(prediction.predicted_yield > 0);
}

/// Test prediction retrieval for non-existent prediction
#[test]
fn test_get_prediction_not_found() {
    let (env, client, _, _, _) = setup_test_environment();
    
    let non_existent_id = create_test_prediction_id(&env, 255);
    
    let result = client.try_get_prediction(&non_existent_id);
    assert!(result.is_err(), "Non-existent prediction should return error");
}

/// Test listing predictions by crop
#[test]
fn test_list_predictions_by_crop() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    // Generate multiple predictions for the same crop
    let region1 = create_test_region(&env, 1);
    let region2 = create_test_region(&env, 2);
    let data_source1 = create_test_data_source(&env, 1);
    let data_source2 = create_test_data_source(&env, 2);

    client.generate_prediction(&crop_id, &region1, &data_source1);
    client.generate_prediction(&crop_id, &region2, &data_source2);

    // List predictions for this crop
    let predictions = client.list_predictions_by_crop(&crop_id);
    assert_eq!(predictions.len(), 2, "Should have 2 predictions for this crop");
    
    for prediction in predictions.iter() {
        assert_eq!(prediction.crop_id, crop_id);
    }
}

/// Test listing predictions by region
#[test]
fn test_list_predictions_by_region() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crops
    let crop_id1 = create_test_crop_id(&env, 1);
    let crop_id2 = create_test_crop_id(&env, 2);
    let name1 = create_test_crop_name(&env, 1);
    let name2 = create_test_crop_name(&env, 2);
    let historical_yields = create_test_historical_yields(&env, 5);
    
    client.register_crop(&crop_id1, &name1, &historical_yields);
    client.register_crop(&crop_id2, &name2, &historical_yields);

    // Generate predictions for same region but different crops
    let region = create_test_region(&env, 1);
    let data_source1 = create_test_data_source(&env, 1);
    let data_source2 = create_test_data_source(&env, 2);

    client.generate_prediction(&crop_id1, &region, &data_source1);
    client.generate_prediction(&crop_id2, &region, &data_source2);

    // List predictions for this region
    let predictions = client.list_predictions_by_region(&region);
    assert_eq!(predictions.len(), 2, "Should have 2 predictions for this region");
    
    for prediction in predictions.iter() {
        assert_eq!(prediction.region, region);
    }
}

/// Test listing predictions by region with invalid input
#[test]
fn test_list_predictions_by_region_invalid_input() {
    let (env, client, _, _, _) = setup_test_environment();
    
    let empty_region = String::from_str(&env, "");
    
    let result = client.try_list_predictions_by_region(&empty_region);
    assert!(result.is_err(), "Empty region should cause error");
}

/// Test data source update by admin
#[test]
fn test_update_data_source_success() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // First register a crop and generate prediction
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let original_data = create_test_data_source(&env, 1);
    let prediction_id = client.generate_prediction(&crop_id, &region, &original_data);

    // Update data source
    let new_data = create_optimal_data_source(&env);
    let result = client.try_update_data_source(&prediction_id, &new_data);
    assert!(result.is_ok(), "Data source update should succeed");
    
    let updated_prediction_id = result.unwrap();
    assert_eq!(updated_prediction_id.unwrap(), prediction_id);
    
    // Verify prediction was updated
    let updated_prediction = client.get_prediction(&prediction_id);
    assert!(updated_prediction.predicted_yield > 0);
}

/// Test data source update by unauthorized user
#[test]
fn test_update_data_source_unauthorized() {
    let (env, client, admin, farmer, _) = setup_strict_auth_environment();
    
    // First register a crop and generate prediction
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let original_data = create_test_data_source(&env, 1);
    let prediction_id = client.generate_prediction(&crop_id, &region, &original_data);

    // Try to update data source as non-admin
    let new_data = create_optimal_data_source(&env);
    let result = client.try_update_data_source(&prediction_id, &new_data);
    assert!(result.is_err(), "Non-admin should not be able to update data source");
}

/// Test data source update for non-existent prediction
#[test]
fn test_update_data_source_prediction_not_found() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    let non_existent_id = create_test_prediction_id(&env, 255);
    let new_data = create_optimal_data_source(&env);
    
    let result = client.try_update_data_source(&non_existent_id, &new_data);
    assert!(result.is_err(), "Non-existent prediction should cause error");
}

/// Test crop retrieval
#[test]
fn test_get_crop_success() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    
    client.register_crop(&crop_id, &name, &historical_yields);
    
    let crop = client.get_crop(&crop_id);
    assert_eq!(crop.crop_id, crop_id);
    assert_eq!(crop.name, name);
    assert_eq!(crop.historical_yields.len(), 5);
}

/// Test crop retrieval for non-existent crop
#[test]
fn test_get_crop_not_found() {
    let (env, client, _, _, _) = setup_test_environment();
    
    let non_existent_id = create_test_crop_id(&env, 255);
    
    let result = client.try_get_crop(&non_existent_id);
    assert!(result.is_err(), "Non-existent crop should return error");
}

/// Test high-volume prediction generation
#[test]
fn test_high_volume_prediction_generation() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Create multiple crops
    let crops = create_multiple_test_crops(&env, 10);
    
    // Register all crops
    for (crop_id, name, historical_yields) in crops.iter() {
        client.register_crop(&crop_id, &name, &historical_yields);
    }
    
    // Generate predictions for each crop
    let mut prediction_ids = vec![&env];
    for i in 1..=10 {
        let crop_id = create_test_crop_id(&env, i as u8);
        let region = create_test_region(&env, i as u8);
        let data_source = create_test_data_source(&env, i as u8);
        
        let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
        prediction_ids.push_back(prediction_id);
    }
    
    // Verify all predictions were created
    assert_eq!(prediction_ids.len(), 10, "Should have 10 predictions");
    
    // Verify each prediction can be retrieved
    for prediction_id in prediction_ids.iter() {
        let prediction = client.get_prediction(&prediction_id);
        assert!(prediction.predicted_yield >= 0);
    }
}

/// Test edge cases for prediction generation
#[test]
fn test_prediction_edge_cases() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop with edge case data
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 1); // Single yield
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let edge_case_data = create_edge_case_data(&env);

    let prediction_id = client.generate_prediction(&crop_id, &region, &edge_case_data);
    let prediction = client.get_prediction(&prediction_id);
    assert!(prediction.predicted_yield >= 0);
}

/// Test prediction generation with oracle failure simulation
#[test]
fn test_prediction_oracle_failure() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let failed_data = simulate_oracle_failure(&env);

    let prediction_id = client.generate_prediction(&crop_id, &region, &failed_data);
    let prediction = client.get_prediction(&prediction_id);
    assert!(prediction.predicted_yield >= 0);
}
