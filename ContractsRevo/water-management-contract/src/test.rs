#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String,
};

use crate::{datatypes::*, WaterManagementContract};

// Test constants
const DAILY_LIMIT: i128 = 5000;
const WEEKLY_LIMIT: i128 = 35000;
const MONTHLY_LIMIT: i128 = 150000;
const EFFICIENT_USAGE_VOLUME: i128 = 2000;
const BASE_REWARD: i128 = 100;

// Helper function to set up test environment
fn setup_test() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(WaterManagementContract, ());
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);

    (env, contract_id, admin, farmer)
}

// Helper function to create test IDs
fn create_usage_id(env: &Env, suffix: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[31] = suffix;
    BytesN::from_array(env, &bytes)
}

fn create_parcel_id(env: &Env, suffix: u8) -> BytesN<32> {
    let mut bytes = [1u8; 32];
    bytes[31] = suffix;
    BytesN::from_array(env, &bytes)
}

fn create_data_hash(env: &Env, suffix: u8) -> BytesN<32> {
    let mut bytes = [2u8; 32];
    bytes[31] = suffix;
    BytesN::from_array(env, &bytes)
}

#[test]
fn test_initialize_contract() {
    let (env, contract_id, admin, _) = setup_test();

    env.as_contract(&contract_id, || {
        let result = WaterManagementContract::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        // Test double initialization fails
        let result2 = WaterManagementContract::initialize(env.clone(), admin.clone());
        assert!(result2.is_err());
    });
}

#[test]
fn test_record_water_usage() {
    let (env, contract_id, admin, farmer) = setup_test();

    env.as_contract(&contract_id, || {
        // Initialize contract
        let _ = WaterManagementContract::initialize(env.clone(), admin.clone());

        let usage_id = create_usage_id(&env, 1);
        let parcel_id = create_parcel_id(&env, 1);
        let data_hash = create_data_hash(&env, 1);
        let volume = 1000i128;

        // Record usage
        let result = WaterManagementContract::record_usage(
            env.clone(),
            usage_id.clone(),
            farmer.clone(),
            parcel_id.clone(),
            volume,
            data_hash,
        );
        assert!(result.is_ok());

        // Verify usage was recorded
        let usage = WaterManagementContract::get_usage(env.clone(), usage_id.clone());
        assert!(usage.is_ok());
        let usage_data = usage.unwrap();
        assert_eq!(usage_data.farmer_id, farmer);
        assert_eq!(usage_data.volume, volume);
    });
}

#[test]
fn test_set_and_get_threshold() {
    let (env, contract_id, admin, _) = setup_test();

    env.as_contract(&contract_id, || {
        // Initialize contract
        let _ = WaterManagementContract::initialize(env.clone(), admin.clone());

        let parcel_id = create_parcel_id(&env, 1);

        // Set threshold
        let result = WaterManagementContract::set_threshold(
            env.clone(),
            admin.clone(),
            parcel_id.clone(),
            DAILY_LIMIT,
            WEEKLY_LIMIT,
            MONTHLY_LIMIT,
        );
        assert!(result.is_ok());

        // Get threshold
        let threshold = WaterManagementContract::get_threshold(env.clone(), parcel_id.clone());
        assert!(threshold.is_ok());
        let threshold_data = threshold.unwrap();
        assert_eq!(threshold_data.daily_limit, DAILY_LIMIT);
        assert_eq!(threshold_data.weekly_limit, WEEKLY_LIMIT);
        assert_eq!(threshold_data.monthly_limit, MONTHLY_LIMIT);
    });
}

#[test]
fn test_incentive_system() {
    let (env, contract_id, admin, farmer) = setup_test();

    env.as_contract(&contract_id, || {
        // Initialize contract
        let _ = WaterManagementContract::initialize(env.clone(), admin.clone());

        let usage_id = create_usage_id(&env, 1);
        let parcel_id = create_parcel_id(&env, 1);
        let data_hash = create_data_hash(&env, 1);
        let volume = EFFICIENT_USAGE_VOLUME; // Efficient usage

        // Set threshold first
        let _ = WaterManagementContract::set_threshold(
            env.clone(),
            admin.clone(),
            parcel_id.clone(),
            DAILY_LIMIT,
            WEEKLY_LIMIT,
            MONTHLY_LIMIT,
        );

        // Record efficient usage
        let _ = WaterManagementContract::record_usage(
            env.clone(),
            usage_id.clone(),
            farmer.clone(),
            parcel_id.clone(),
            volume,
            data_hash,
        );

        // Issue incentive
        let result = WaterManagementContract::issue_incentive(
            env.clone(),
            usage_id.clone(),
            BASE_REWARD,
        );
        assert!(result.is_ok());

        // Verify incentive was created
        let incentive = WaterManagementContract::get_incentive(env.clone(), usage_id.clone());
        assert!(incentive.is_ok());
        let incentive_data = incentive.unwrap();
        assert_eq!(incentive_data.farmer_id, farmer);
        assert!(incentive_data.reward_amount > 0);
    });
}

#[test]
fn test_usage_report() {
    let (env, contract_id, admin, farmer) = setup_test();

    env.as_contract(&contract_id, || {
        // Initialize contract
        let _ = WaterManagementContract::initialize(env.clone(), admin.clone());

        let parcel_id = create_parcel_id(&env, 1);
        let data_hash = create_data_hash(&env, 1);

        // Record multiple usages
        for i in 1..=3 {
            let usage_id = create_usage_id(&env, i);
            let volume = 1000i128 * i as i128;

            let _ = WaterManagementContract::record_usage(
                env.clone(),
                usage_id,
                farmer.clone(),
                parcel_id.clone(),
                volume,
                data_hash.clone(),
            );
        }

        // Get usage report
        let current_time = env.ledger().timestamp();
        let report = WaterManagementContract::get_usage_report(
            env.clone(),
            farmer.clone(),
            Some(parcel_id.clone()),
            current_time - 86400, // 24 hours ago
            current_time + 86400, // 24 hours from now
        );

        assert!(report.is_ok());
        let report_data = report.unwrap();
        assert_eq!(report_data.farmer_id, farmer);
        assert_eq!(report_data.total_usage, 6000i128); // 1000 + 2000 + 3000
    });
}

#[test]
fn test_alert_generation() {
    let (env, contract_id, admin, farmer) = setup_test();

    env.as_contract(&contract_id, || {
        // Initialize contract
        let _ = WaterManagementContract::initialize(env.clone(), admin.clone());

        let alert_id = create_usage_id(&env, 99); // Reuse function for alert ID
        let parcel_id = create_parcel_id(&env, 1);
        let message = String::from_str(&env, "Test alert message");

        // Generate alert
        let result = WaterManagementContract::generate_alert(
            env.clone(),
            alert_id.clone(),
            farmer.clone(),
            parcel_id.clone(),
            AlertType::ExcessiveUsage,
            message.clone(),
        );
        assert!(result.is_ok());

        // Verify alert was created
        let alert = WaterManagementContract::get_alert(env.clone(), alert_id.clone());
        assert!(alert.is_ok());
        let alert_data = alert.unwrap();
        assert_eq!(alert_data.farmer_id, farmer);
        assert_eq!(alert_data.message, message);
        assert!(!alert_data.resolved);

        // Resolve alert
        let resolve_result = WaterManagementContract::resolve_alert(
            env.clone(),
            alert_id.clone(),
            farmer.clone(),
        );
        assert!(resolve_result.is_ok());

        // Verify alert is resolved
        let resolved_alert = WaterManagementContract::get_alert(env.clone(), alert_id);
        assert!(resolved_alert.is_ok());
        assert!(resolved_alert.unwrap().resolved);
    });
}

#[test]
fn test_farmer_rewards_calculation() {
    let (env, contract_id, admin, farmer) = setup_test();

    env.as_contract(&contract_id, || {
        // Initialize contract
        let _ = WaterManagementContract::initialize(env.clone(), admin.clone());

        let parcel_id = create_parcel_id(&env, 1);
        let data_hash = create_data_hash(&env, 1);
        let daily_limit = 5000i128;

        // Set threshold
        let _ = WaterManagementContract::set_threshold(
            env.clone(),
            admin.clone(),
            parcel_id.clone(),
            daily_limit,
            35000i128,
            150000i128,
        );

        // Record efficient usage and issue incentives
        let mut total_expected_rewards = 0i128;
        for i in 1..=2 {
            let usage_id = create_usage_id(&env, i);
            let volume = EFFICIENT_USAGE_VOLUME; // Efficient usage

            let _ = WaterManagementContract::record_usage(
                env.clone(),
                usage_id.clone(),
                farmer.clone(),
                parcel_id.clone(),
                volume,
                data_hash.clone(),
            );

            let _ = WaterManagementContract::issue_incentive(
                env.clone(),
                usage_id.clone(),
                BASE_REWARD,
            );

            // Get actual reward amount
            let incentive = WaterManagementContract::get_incentive(env.clone(), usage_id).unwrap();
            total_expected_rewards += incentive.reward_amount;
        }

        // Calculate farmer rewards
        let current_time = env.ledger().timestamp();
        let rewards = WaterManagementContract::calculate_farmer_rewards(
            env.clone(),
            farmer.clone(),
            current_time - 86400,
            current_time + 86400,
        );

        assert!(rewards.is_ok());
        assert_eq!(rewards.unwrap(), total_expected_rewards);
    });
}
