#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Ledger as _},
    vec, Address, BytesN, Env, String, Vec,
};

use crate::{
    CropYieldPredictionContractClient,
    types::{CropYieldError, YieldPrediction, YieldReport, MarketInsight},
    reporting::ReportingService,
};

use super::utils::*;

/// Test farmer report generation with high yield prediction
#[test]
fn test_generate_farmer_report_high_yield() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop and generate prediction
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let optimal_data = create_optimal_data_source(&env);
    let prediction_id = client.generate_prediction(&crop_id, &region, &optimal_data);
    let prediction = client.get_prediction(&prediction_id);
    let crop = client.get_crop(&crop_id);

    // Generate farmer report
    let report = ReportingService::generate_farmer_report(&env, &prediction, &crop);
    
    // Verify report structure
    assert_eq!(report.crop_name, crop.name);
    assert_eq!(report.region, prediction.region);
    assert_eq!(report.predicted_yield, prediction.predicted_yield);
    assert!(report.recommendations.len() > 0);
    assert!(report.report_date > 0);
    
    // High yield should have specific recommendations
    if prediction.predicted_yield > 1000 {
        assert!(report.recommendations.len() > 0, "High yield should have recommendations");
    }
}

/// Test farmer report generation with low yield prediction
#[test]
fn test_generate_farmer_report_low_yield() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop and generate prediction
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let poor_data = create_poor_data_source(&env);
    let prediction_id = client.generate_prediction(&crop_id, &region, &poor_data);
    let prediction = client.get_prediction(&prediction_id);
    let crop = client.get_crop(&crop_id);

    // Generate farmer report
    let report = ReportingService::generate_farmer_report(&env, &prediction, &crop);
    
    // Verify report structure
    assert_eq!(report.crop_name, crop.name);
    assert_eq!(report.region, prediction.region);
    assert_eq!(report.predicted_yield, prediction.predicted_yield);
    assert!(report.recommendations.len() > 0);
    assert!(report.report_date > 0);
    
    // Low yield should have specific recommendations
    if prediction.predicted_yield < 500 {
        assert!(report.recommendations.len() > 0, "Low yield should have recommendations");
    }
}

/// Test farmer report generation with moderate yield prediction
#[test]
fn test_generate_farmer_report_moderate_yield() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop and generate prediction
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let moderate_data = create_test_data_source(&env, 1);
    let prediction_id = client.generate_prediction(&crop_id, &region, &moderate_data);
    let prediction = client.get_prediction(&prediction_id);
    let crop = client.get_crop(&crop_id);

    // Generate farmer report
    let report = ReportingService::generate_farmer_report(&env, &prediction, &crop);
    
    // Verify report structure
    assert_eq!(report.crop_name, crop.name);
    assert_eq!(report.region, prediction.region);
    assert_eq!(report.predicted_yield, prediction.predicted_yield);
    assert!(report.recommendations.len() > 0);
    assert!(report.report_date > 0);
    
    // Moderate yield should have specific recommendations
    if prediction.predicted_yield >= 500 && prediction.predicted_yield <= 1000 {
        assert!(report.recommendations.len() > 0, "Moderate yield should have recommendations");
    }
}

/// Test buyer market insights generation
#[test]
fn test_generate_buyer_insights() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register multiple crops
    let crops = create_multiple_test_crops(&env, 3);
    for (crop_id, name, historical_yields) in crops.iter() {
        client.register_crop(&crop_id, &name, &historical_yields);
    }

    let region = create_test_region(&env, 1);
    let mut predictions = vec![&env];
    
    // Generate predictions for each crop in the same region
    for i in 1..=3 {
        let crop_id = create_test_crop_id(&env, i as u8);
        let data_source = create_test_data_source(&env, i as u8);
        let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
        let prediction = client.get_prediction(&prediction_id);
        predictions.push_back(prediction);
    }

    // Generate buyer insights
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    // Verify insights structure
    assert!(insights.len() > 0, "Should generate insights for the region");
    
    for insight in insights.iter() {
        assert_eq!(insight.region, region);
        assert!(insight.expected_supply >= 0);
        assert!(insight.price_trend.len() > 0);
        assert!(insight.buying_recommendation.len() > 0);
    }
}

/// Test buyer market insights with multiple regions
#[test]
fn test_generate_buyer_insights_multiple_regions() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let mut predictions = vec![&env];
    
    // Generate predictions for different regions
    for i in 1..=3 {
        let region = create_test_region(&env, i as u8);
        let data_source = create_test_data_source(&env, i as u8);
        let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
        let prediction = client.get_prediction(&prediction_id);
        predictions.push_back(prediction);
    }

    // Generate insights for specific region
    let target_region = create_test_region(&env, 1);
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, target_region.clone());
    
    // Should only include insights for the target region
    assert_eq!(insights.len(), 1, "Should have one insight for the target region");
    
    let insight = insights.get(0).unwrap();
    assert_eq!(insight.region, target_region);
}

/// Test buyer market insights with high supply scenario
#[test]
fn test_generate_buyer_insights_high_supply() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let optimal_data = create_optimal_data_source(&env);
    let prediction_id = client.generate_prediction(&crop_id, &region, &optimal_data);
    let prediction = client.get_prediction(&prediction_id);
    
    let predictions = vec![&env, prediction];
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    assert_eq!(insights.len(), 1);
    let insight = insights.get(0).unwrap();
    
    // High supply should have specific recommendations
    if insight.expected_supply > 1000 {
        assert!(insight.buying_recommendation.len() > 0, "High supply should have buying recommendations");
    }
}

/// Test buyer market insights with low supply scenario
#[test]
fn test_generate_buyer_insights_low_supply() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let poor_data = create_poor_data_source(&env);
    let prediction_id = client.generate_prediction(&crop_id, &region, &poor_data);
    let prediction = client.get_prediction(&prediction_id);
    
    let predictions = vec![&env, prediction];
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    assert_eq!(insights.len(), 1);
    let insight = insights.get(0).unwrap();
    
    // Low supply should have specific recommendations
    if insight.expected_supply < 500 {
        assert!(insight.buying_recommendation.len() > 0, "Low supply should have buying recommendations");
    }
}

/// Test buyer market insights with stable supply scenario
#[test]
fn test_generate_buyer_insights_stable_supply() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let moderate_data = create_test_data_source(&env, 1);
    let prediction_id = client.generate_prediction(&crop_id, &region, &moderate_data);
    let prediction = client.get_prediction(&prediction_id);
    
    let predictions = vec![&env, prediction];
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    assert_eq!(insights.len(), 1);
    let insight = insights.get(0).unwrap();
    
    // Stable supply should have specific recommendations
    if insight.expected_supply >= 500 && insight.expected_supply <= 1000 {
        assert!(insight.buying_recommendation.len() > 0, "Stable supply should have buying recommendations");
    }
}

/// Test report generation with empty predictions
#[test]
fn test_generate_buyer_insights_empty_predictions() {
    let (env, _, _, _, _) = setup_test_environment();
    
    let region = create_test_region(&env, 1);
    let empty_predictions = vec![&env];
    
    let insights = ReportingService::generate_buyer_insights(&env, &empty_predictions, region);
    
    // Empty predictions should result in empty insights
    assert_eq!(insights.len(), 0, "Empty predictions should result in empty insights");
}

/// Test report generation with predictions from different regions
#[test]
fn test_generate_buyer_insights_different_regions() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let mut predictions = vec![&env];
    
    // Generate predictions for different regions
    for i in 1..=3 {
        let region = create_test_region(&env, i as u8);
        let data_source = create_test_data_source(&env, i as u8);
        let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
        let prediction = client.get_prediction(&prediction_id);
        predictions.push_back(prediction);
    }

    // Generate insights for specific region
    let target_region = create_test_region(&env, 2);
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, target_region.clone());
    
    // Should only include insights for the target region
    assert_eq!(insights.len(), 1, "Should have one insight for the target region");
    
    let insight = insights.get(0).unwrap();
    assert_eq!(insight.region, target_region);
}

/// Test report generation with multiple crops in same region
#[test]
fn test_generate_buyer_insights_multiple_crops_same_region() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register multiple crops
    let crops = create_multiple_test_crops(&env, 3);
    for (crop_id, name, historical_yields) in crops.iter() {
        client.register_crop(&crop_id, &name, &historical_yields);
    }

    let region = create_test_region(&env, 1);
    let mut predictions = vec![&env];
    
    // Generate predictions for each crop in the same region
    for i in 1..=3 {
        let crop_id = create_test_crop_id(&env, i as u8);
        let data_source = create_test_data_source(&env, i as u8);
        let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
        let prediction = client.get_prediction(&prediction_id);
        predictions.push_back(prediction);
    }

    // Generate insights for the region
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    // Should have insights for all crops in the region
    assert_eq!(insights.len(), 3, "Should have insights for all 3 crops in the region");
    
    for insight in insights.iter() {
        assert_eq!(insight.region, region);
        assert!(insight.expected_supply >= 0);
        assert!(insight.price_trend.len() > 0);
        assert!(insight.buying_recommendation.len() > 0);
    }
}

/// Test report generation with extreme yield predictions
#[test]
fn test_generate_reports_extreme_yields() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let extreme_data = create_extreme_data_source(&env);
    let prediction_id = client.generate_prediction(&crop_id, &region, &extreme_data);
    let prediction = client.get_prediction(&prediction_id);
    let crop = client.get_crop(&crop_id);

    // Generate farmer report for extreme conditions
    let report = ReportingService::generate_farmer_report(&env, &prediction, &crop);
    
    // Verify report handles extreme conditions
    assert_eq!(report.predicted_yield, prediction.predicted_yield);
    assert!(report.recommendations.len() > 0, "Extreme conditions should have recommendations");
    
    // Generate buyer insights for extreme conditions
    let predictions = vec![&env, prediction];
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    assert_eq!(insights.len(), 1);
    let insight = insights.get(0).unwrap();
    assert!(insight.buying_recommendation.len() > 0, "Extreme conditions should have buying recommendations");
}

/// Test report generation with zero yield prediction
#[test]
fn test_generate_reports_zero_yield() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop with minimal historical data
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let minimal_yields = vec![&env, 0i128]; // Single zero yield
    client.register_crop(&crop_id, &name, &minimal_yields);

    let region = create_test_region(&env, 1);
    let poor_data = create_poor_data_source(&env);
    let prediction_id = client.generate_prediction(&crop_id, &region, &poor_data);
    let prediction = client.get_prediction(&prediction_id);
    let crop = client.get_crop(&crop_id);

    // Generate farmer report for zero yield
    let report = ReportingService::generate_farmer_report(&env, &prediction, &crop);
    
    // Verify report handles zero yield
    assert_eq!(report.predicted_yield, prediction.predicted_yield);
    assert!(report.recommendations.len() > 0, "Zero yield should have recommendations");
    
    // Generate buyer insights for zero yield
    let predictions = vec![&env, prediction];
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    assert_eq!(insights.len(), 1);
    let insight = insights.get(0).unwrap();
    assert!(insight.buying_recommendation.len() > 0, "Zero yield should have buying recommendations");
}

/// Test report generation with high-volume predictions
#[test]
fn test_generate_reports_high_volume() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register multiple crops
    let crops = create_multiple_test_crops(&env, 10);
    for (crop_id, name, historical_yields) in crops.iter() {
        client.register_crop(&crop_id, &name, &historical_yields);
    }

    let region = create_test_region(&env, 1);
    let mut predictions = vec![&env];
    
    // Generate many predictions
    for i in 1..=10 {
        let crop_id = create_test_crop_id(&env, i as u8);
        let data_source = create_test_data_source(&env, i as u8);
        let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
        let prediction = client.get_prediction(&prediction_id);
        predictions.push_back(prediction);
    }

    // Generate insights for high volume
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    // Should handle high volume efficiently
    assert_eq!(insights.len(), 10, "Should generate insights for all 10 crops");
    
    for insight in insights.iter() {
        assert_eq!(insight.region, region);
        assert!(insight.expected_supply >= 0);
        assert!(insight.price_trend.len() > 0);
        assert!(insight.buying_recommendation.len() > 0);
    }
}

/// Test report generation with mixed yield scenarios
#[test]
fn test_generate_reports_mixed_yield_scenarios() {
    let (env, client, admin, _, _) = setup_test_environment();
    
    // Register crop
    let crop_id = create_test_crop_id(&env, 1);
    let name = create_test_crop_name(&env, 1);
    let historical_yields = create_test_historical_yields(&env, 5);
    client.register_crop(&crop_id, &name, &historical_yields);

    let region = create_test_region(&env, 1);
    let mut predictions = vec![&env];
    
    // Generate predictions with different conditions
    let data_sources = vec![
        &env,
        create_optimal_data_source(&env),
        create_poor_data_source(&env),
        create_extreme_data_source(&env),
        create_test_data_source(&env, 1),
    ];
    
    for data_source in data_sources.iter() {
        let prediction_id = client.generate_prediction(&crop_id, &region, &data_source);
        let prediction = client.get_prediction(&prediction_id);
        predictions.push_back(prediction);
    }

    // Generate insights for mixed scenarios
    let insights = ReportingService::generate_buyer_insights(&env, &predictions, region.clone());
    
    // Should handle mixed scenarios
    assert_eq!(insights.len(), 4, "Should generate insights for all 4 predictions");
    
    for insight in insights.iter() {
        assert_eq!(insight.region, region);
        assert!(insight.expected_supply >= 0);
        assert!(insight.price_trend.len() > 0);
        assert!(insight.buying_recommendation.len() > 0);
    }
}
