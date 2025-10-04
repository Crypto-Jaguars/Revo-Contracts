#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

use crate::{WaterManagementContract, WaterManagementContractClient};

use super::utils::*;

/// Test water usage recording and validation
#[test]
fn test_record_water_usage_success() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    // Initialize contract
    client.initialize(&admin);

    let usage_id = create_test_usage_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);
    let volume = 1000i128;

    // Record usage
    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    assert!(result.is_ok());

    // Verify usage was recorded
    let usage_data = client.get_usage(&usage_id);
    assert_eq!(usage_data.farmer_id, farmer);
    assert_eq!(usage_data.volume, volume);
    assert_eq!(usage_data.parcel_id, parcel_id);
    assert_eq!(usage_data.data_hash, data_hash);
}

#[test]
fn test_record_water_usage_duplicate_id() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let usage_id = create_test_usage_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);
    let volume = 1000i128;

    // Record first usage
    let result1 = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    assert!(result1.is_ok());

    // Try to record duplicate usage ID
    let result2 = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    assert!(result2.is_err());
}

#[test]
fn test_record_water_usage_invalid_volume() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let usage_id = create_test_usage_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);

    // Test negative volume
    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &(-100i128), &data_hash);
    assert!(result.is_err());

    // Test zero volume
    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &0i128, &data_hash);
    assert!(result.is_err());

    // Test excessive volume (> 100,000 liters)
    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &150_000i128, &data_hash);
    assert!(result.is_err());
}

#[test]
fn test_record_water_usage_invalid_data_hash() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let usage_id = create_test_usage_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let empty_hash = BytesN::from_array(&env, &[0u8; 32]);
    let volume = 1000i128;

    // Test empty data hash
    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &empty_hash);
    assert!(result.is_err());
}

#[test]
fn test_record_water_usage_invalid_identifiers() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let empty_id = BytesN::from_array(&env, &[0u8; 32]);
    let data_hash = create_test_data_hash(&env, 1);
    let volume = 1000i128;

    // Test empty usage ID
    let result = client.try_record_usage(&empty_id, &farmer, &empty_id, &volume, &data_hash);
    assert!(result.is_err());

    // Test empty parcel ID
    let usage_id = create_test_usage_id(&env, 1);
    let result = client.try_record_usage(&usage_id, &farmer, &empty_id, &volume, &data_hash);
    assert!(result.is_err());
}

#[test]
fn test_get_usage_not_found() {
    let (env, client, admin, _) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let non_existent_id = create_test_usage_id(&env, 255);

    let result = client.try_get_usage(&non_existent_id);
    assert!(result.is_err());
}

#[test]
fn test_get_farmer_usages() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);

    // Record multiple usages for the same farmer
    for i in 1..=3 {
        let usage_id = create_test_usage_id(&env, i);
        let volume = 1000i128 * i as i128;
        let _ = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    }

    // Get all usages for the farmer
    let farmer_usages = client.get_farmer_usages(&farmer);
    assert_eq!(farmer_usages.len(), 3);

    // Verify all usages belong to the farmer
    for usage in farmer_usages.iter() {
        assert_eq!(usage.farmer_id, farmer);
    }
}

#[test]
fn test_get_parcel_usages() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);

    // Record multiple usages for the same parcel
    for i in 1..=3 {
        let usage_id = create_test_usage_id(&env, i);
        let volume = 1000i128 * i as i128;
        let _ = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    }

    // Get all usages for the parcel
    let parcel_usages = client.get_parcel_usages(&parcel_id);
    assert_eq!(parcel_usages.len(), 3);

    // Verify all usages belong to the parcel
    for usage in parcel_usages.iter() {
        assert_eq!(usage.parcel_id, parcel_id);
    }
}

#[test]
fn test_usage_report_farmer_wide() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id1 = create_test_parcel_id(&env, 1);
    let parcel_id2 = create_test_parcel_id(&env, 2);
    let data_hash = create_test_data_hash(&env, 1);

    // Record usages across multiple parcels
    for i in 1..=3 {
        let usage_id = create_test_usage_id(&env, i);
        let parcel_id = if i % 2 == 0 {
            parcel_id1.clone()
        } else {
            parcel_id2.clone()
        };
        let volume = 1000i128 * i as i128;
        let _ = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    }

    // Get farmer-wide usage report
    let current_time = env.ledger().timestamp();
    let start_time = if current_time > 86400 {
        current_time - 86400
    } else {
        0
    };
    let end_time = current_time + 86400;

    let report = client.get_usage_report(
        &farmer,
        &None, // No specific parcel - farmer-wide report
        &start_time,
        &end_time,
    );

    assert_eq!(report.farmer_id, farmer);
    assert_eq!(report.is_farmer_wide, true);
    assert_eq!(report.total_usage, 6000i128); // 1000 + 2000 + 3000
}

#[test]
fn test_usage_report_parcel_specific() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id1 = create_test_parcel_id(&env, 1);
    let parcel_id2 = create_test_parcel_id(&env, 2);
    let data_hash = create_test_data_hash(&env, 1);

    // Record usages across multiple parcels
    for i in 1..=3 {
        let usage_id = create_test_usage_id(&env, i);
        let parcel_id = if i % 2 == 0 {
            parcel_id1.clone()
        } else {
            parcel_id2.clone()
        };
        let volume = 1000i128 * i as i128;
        let _ = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    }

    // Get parcel-specific usage report
    let current_time = env.ledger().timestamp();
    let start_time = if current_time > 86400 {
        current_time - 86400
    } else {
        0
    };
    let end_time = current_time + 86400;

    let report =
        client.get_usage_report(&farmer, &Some(parcel_id1.clone()), &start_time, &end_time);

    assert_eq!(report.farmer_id, farmer);
    assert_eq!(report.parcel_id, parcel_id1);
    assert_eq!(report.is_farmer_wide, false);
    // Only usages for parcel_id1 (even-numbered iterations: 2000)
    assert_eq!(report.total_usage, 2000i128);
}

#[test]
fn test_usage_report_invalid_time_range() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let current_time = env.ledger().timestamp();
    let start_time = current_time + 1000; // Start time in the future
    let end_time = current_time;

    let result = client.try_get_usage_report(&farmer, &None, &start_time, &end_time);
    assert!(result.is_err());
}

#[test]
fn test_usage_report_empty_period() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let current_time = env.ledger().timestamp();
    let start_time = current_time + 1000; // Future start time
    let end_time = current_time + 2000; // Future end time

    let report = client.get_usage_report(&farmer, &None, &start_time, &end_time);

    assert_eq!(report.farmer_id, farmer);
    assert_eq!(report.total_usage, 0i128);
    assert_eq!(report.efficiency_score, 0u32);
}

#[test]
fn test_high_volume_usage_recording() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);

    // Test recording many usage records to test scalability
    for i in 1..=50 {
        let usage_id = create_test_usage_id(&env, i);
        let volume = 100i128 + (i as i128 * 10);
        let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
        assert!(result.is_ok());
    }

    // Verify all records were stored
    let farmer_usages = client.get_farmer_usages(&farmer);
    assert_eq!(farmer_usages.len(), 50);

    let parcel_usages = client.get_parcel_usages(&parcel_id);
    assert_eq!(parcel_usages.len(), 50);
}

#[test]
fn test_oracle_data_integration_simulation() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let usage_id = create_test_usage_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let volume = 1000i128;

    // Simulate different types of sensor data hashes
    let sensor_data_hashes = [
        create_test_data_hash(&env, 1), // Normal sensor reading
        create_test_data_hash(&env, 2), // Temperature sensor
        create_test_data_hash(&env, 3), // Humidity sensor
        create_test_data_hash(&env, 4), // Flow rate sensor
    ];

    for (i, data_hash) in sensor_data_hashes.iter().enumerate() {
        let usage_id = create_test_usage_id(&env, (i + 1) as u8);
        let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, data_hash);
        assert!(result.is_ok());
    }

    // Verify all sensor data was recorded
    let farmer_usages = client.get_farmer_usages(&farmer);
    assert_eq!(farmer_usages.len(), 4);
}

#[test]
fn test_usage_timestamp_validation() {
    let (env, client, admin, farmer) = setup_test_environment();
    env.mock_all_auths();

    client.initialize(&admin);

    let usage_id = create_test_usage_id(&env, 1);
    let parcel_id = create_test_parcel_id(&env, 1);
    let data_hash = create_test_data_hash(&env, 1);
    let volume = 1000i128;

    // Record usage normally
    let result = client.try_record_usage(&usage_id, &farmer, &parcel_id, &volume, &data_hash);
    assert!(result.is_ok());

    // Verify timestamp is set correctly
    let usage = client.get_usage(&usage_id);
    let current_time = env.ledger().timestamp();
    assert!(usage.timestamp <= current_time);
    assert!(current_time - usage.timestamp < 10); // Should be very recent
}
