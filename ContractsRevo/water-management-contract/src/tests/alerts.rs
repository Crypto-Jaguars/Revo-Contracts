#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, BytesN, Env, String, Vec,
};

use crate::{
    datatypes::*,
    WaterManagementContract,
    WaterManagementContractClient,
};

use super::utils::*;

/// Test alert generation and threshold-based triggers
#[test]
fn test_generate_alert_success() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate alert
    let result = client.try_generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );
    assert!(result.is_ok());

    // Verify alert was created
    let alert = client.get_alert(&alert_id);
    assert_eq!(alert.farmer_id, farmer);
    assert_eq!(alert.parcel_id, parcel_id);
    assert_eq!(alert.message, message);
    assert_eq!(alert.alert_type, AlertType::ExcessiveUsage);
    assert!(!alert.resolved);
}

#[test]
fn test_generate_alert_duplicate_id() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate first alert
    let result1 = client.try_generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );
    assert!(result1.is_ok());

    // Try to generate duplicate alert
    let result2 = client.try_generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ThresholdExceeded,
        &message,
    );
    assert!(result2.is_err());
}

#[test]
fn test_generate_alert_empty_message() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let empty_message = String::from_str(&env, "");

    // Try to generate alert with empty message
    let result = client.try_generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &empty_message,
    );
    assert!(result.is_err());
}

#[test]
fn test_generate_alert_invalid_identifiers() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let empty_id = BytesN::from_array(&env, &[0u8; 32]);
    let message = String::from_str(&env, "Test message");

    // Test empty alert ID
    let result = client.try_generate_alert(
        &empty_id,
        &farmer,
        &empty_id,
        &AlertType::ExcessiveUsage,
        &message,
    );
    assert!(result.is_err());
}

#[test]
fn test_resolve_alert_success() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate alert
    client.generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );

    // Resolve alert
    let result = client.try_resolve_alert(&alert_id, &farmer);
    assert!(result.is_ok());

    // Verify alert is resolved
    let resolved_alert = client.get_alert(&alert_id);
    assert!(resolved_alert.resolved);
}

#[test]
fn test_resolve_alert_unauthorized() {
    let (env, client, admin, farmer) = setup_test_environment();
    let unauthorized_farmer = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate alert
    client.generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );

    // Try to resolve alert by unauthorized farmer
    let result = client.try_resolve_alert(&alert_id, &unauthorized_farmer);
    assert!(result.is_err());
}

#[test]
fn test_resolve_alert_by_admin() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate alert
    client.generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );

    // Resolve alert by admin
    let result = client.try_resolve_alert(&alert_id, &admin);
    assert!(result.is_ok());

    // Verify alert is resolved
    let resolved_alert = client.get_alert(&alert_id);
    assert!(resolved_alert.resolved);
}

#[test]
fn test_resolve_alert_not_found() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let non_existent_alert_id = create_test_alert_id(&env, 255);

    // Try to resolve non-existent alert
    let result = client.try_resolve_alert(&non_existent_alert_id, &farmer);
    assert!(result.is_err());
}

#[test]
fn test_get_farmer_alerts() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate multiple alerts
    for i in 1..=3 {
        let alert_id = create_test_alert_id(&env, i);
        let alert_type = match i {
            1 => AlertType::ExcessiveUsage,
            2 => AlertType::ThresholdExceeded,
            3 => AlertType::SensorMalfunction,
            _ => AlertType::EfficiencyAlert,
        };

        client.generate_alert(
            &alert_id,
            &farmer,
            &parcel_id,
            &alert_type,
            &message,
        );
    }

    // Get all alerts for farmer
    let alerts = client.get_farmer_alerts(&farmer, &true); // Include resolved
    assert_eq!(alerts.len(), 3);

    // Verify all alerts belong to the farmer
    for alert in alerts.iter() {
        assert_eq!(alert.farmer_id, farmer);
    }
}

#[test]
fn test_get_farmer_alerts_exclude_resolved() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate alerts
    for i in 1..=3 {
        let alert_id = create_test_alert_id(&env, i);
        client.generate_alert(
            &alert_id,
            &farmer,
            &parcel_id,
            &AlertType::ExcessiveUsage,
            &message,
        );
    }

    // Resolve one alert
    let alert_id_to_resolve = create_test_alert_id(&env, 1);
    client.resolve_alert(&alert_id_to_resolve, &farmer);

    // Get unresolved alerts only
    let unresolved_alerts = client.get_farmer_alerts(&farmer, &false);
    assert_eq!(unresolved_alerts.len(), 2);

    // Get all alerts (including resolved)
    let all_alerts = client.get_farmer_alerts(&farmer, &true);
    assert_eq!(all_alerts.len(), 3);
}

#[test]
fn test_threshold_exceeded_alert_trigger() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);

    // Set threshold
    client.set_threshold(&admin, &parcel_id, &5000i128, &35000i128, &150000i128);

    // Record usage that exceeds daily limit
    let usage_id = create_test_usage_id(&env, 1);
    let volume = 6000i128; // Exceeds daily limit of 5000

    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    assert!(result.is_ok());

    // The usage recording should trigger automatic alert generation
    // Check if alerts were generated for this farmer
    let alerts = client.get_farmer_alerts(&farmer, &true);
    // Note: The exact number depends on implementation
    // At minimum, there should be alerts for threshold exceeded and excessive usage
    assert!(alerts.len() >= 1);
}

#[test]
fn test_excessive_single_usage_alert() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);

    // Set threshold
    client.set_threshold(&admin, &parcel_id, &5000i128, &35000i128, &150000i128);

    // Record usage that is more than 50% of daily limit in one record
    let usage_id = create_test_usage_id(&env, 1);
    let volume = 3000i128; // More than 50% of 5000 (2500)

    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    assert!(result.is_ok());

    // Check if excessive usage alert was generated
    let alerts = client.get_farmer_alerts(&farmer, &true);
    let mut excessive_count = 0;
    for alert in alerts.iter() {
        if alert.alert_type == AlertType::ExcessiveUsage {
            excessive_count += 1;
        }
    }
    
    assert!(excessive_count >= 1);
}

#[test]
fn test_weekly_threshold_exceeded_alert() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);

    // Set threshold
    client.set_threshold(&admin, &parcel_id, &5000i128, &35000i128, &150000i128);

    // Record multiple usages that together exceed weekly limit
    for i in 1..=8 {
        let usage_id = create_test_usage_id(&env, i);
        let volume = 5000i128; // Each usage is at daily limit
        let _ = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    }

    // Check if weekly threshold exceeded alert was generated
    let alerts = client.get_farmer_alerts(&farmer, &true);
    let weekly_count = alerts
        .iter()
        .filter(|a| a.alert_type == AlertType::ThresholdExceeded)
        .count();
    assert!(
        weekly_count >= 1,
        "Expected at least one weekly ThresholdExceeded alert"
    );
}

#[test]
fn test_alert_types_coverage() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Test all alert types
    let alert_types = [
        AlertType::ExcessiveUsage,
        AlertType::ThresholdExceeded,
        AlertType::SensorMalfunction,
        AlertType::EfficiencyAlert,
    ];

    for (i, alert_type) in alert_types.iter().enumerate() {
        let alert_id = create_test_alert_id(&env, (i + 1) as u8);
        
        let result = client.try_generate_alert(
            &alert_id,
            &farmer,
            &parcel_id,
            alert_type,
            &message,
        );
        assert!(result.is_ok());

        // Verify alert was created with correct type
        let alert = client.get_alert(&alert_id);
        assert_eq!(alert.alert_type, *alert_type);
    }
}

#[test]
fn test_alert_timestamp_validation() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate alert
    client.generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );

    // Verify timestamp is set correctly
    let alert = client.get_alert(&alert_id);
    let current_time = env.ledger().timestamp();
    assert!(alert.timestamp <= current_time);
    assert!(current_time - alert.timestamp < 10); // Should be very recent
}

#[test]
fn test_alert_resolution_timestamp() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let alert_id = create_test_alert_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Generate alert
    client.generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );

    let alert_before_resolution = client.get_alert(&alert_id);
    let creation_time = alert_before_resolution.timestamp;

    // Resolve alert
    client.resolve_alert(&alert_id, &farmer);

    let alert_after_resolution = client.get_alert(&alert_id);
    assert!(alert_after_resolution.resolved);
    assert_eq!(alert_after_resolution.timestamp, creation_time); // Timestamp should not change
}

#[test]
fn test_alert_edge_cases() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let message = String::from_str(&env, "Test alert message");

    // Test resolving already resolved alert
    let alert_id = create_test_alert_id(&env, 1);
    
    client.generate_alert(
        &alert_id,
        &farmer,
        &parcel_id,
        &AlertType::ExcessiveUsage,
        &message,
    );

    // Resolve first time
    let result1 = client.try_resolve_alert(&alert_id, &farmer);
    assert!(result1.is_ok());

    // Try to resolve again
    let result2 = client.try_resolve_alert(&alert_id, &farmer);
    assert!(result2.is_ok()); // Should succeed (idempotent operation)
}
