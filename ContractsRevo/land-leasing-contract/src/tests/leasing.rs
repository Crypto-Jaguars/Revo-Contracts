#![cfg(test)]

use super::utils::*;
use crate::*;
use soroban_sdk::{testutils::Address as _, Bytes, String};

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, _, _, _) = create_test_accounts(&env);

    client.initialize(&admin);

    // Verify admin is set - use as_contract to access storage
    let is_admin_result = env.as_contract(&contract_id, || utils::is_admin(&env, &admin));
    assert!(is_admin_result);
}

#[test]
fn test_create_lease_agreement() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"test_land_1");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Test Farm Location");
    let data_bytes = Bytes::from_slice(&env, b"land_data_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100,  // 100 hectares
        &12,   // 12 months
        &1000, // 1000 units per month
        &data_hash,
    );

    // Verify lease was created
    let lease_details = client.get_lease_details(&lease_id);
    assert!(lease_details.is_some());

    let lease = lease_details.unwrap();
    assert_eq!(lease.lessor_id, lessor);
    assert_eq!(lease.lessee_id, lessee);
    assert_eq!(lease.land_id, land_id);
    assert_eq!(lease.duration, 12);
    assert_eq!(lease.payment_amount, 1000);
    assert_eq!(lease.status, String::from_str(&env, "Active"));
}

#[test]
fn test_terminate_lease() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"test_land_3");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Test Farm Location 3");
    let data_bytes = Bytes::from_slice(&env, b"land_data_hash_3");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &75, &8, &800, &data_hash,
    );

    // Terminate lease
    let termination_result = client.terminate_lease(&lease_id, &lessor);
    assert!(termination_result);

    // Verify lease status
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Terminated"));
}

#[test]
#[should_panic(expected = "Duration must be greater than 0")]
fn test_create_lease_invalid_duration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"test_land_invalid");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Invalid Test Location");
    let data_bytes = Bytes::from_slice(&env, b"invalid_data_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    // This should panic due to zero duration
    client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &0, // Invalid duration
        &1000, &data_hash,
    );
}

#[test]
fn test_simple_functionality() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    // Initialize
    client.initialize(&admin);

    // Create a multi-month lease so one payment doesn't complete it
    let land_bytes = Bytes::from_slice(&env, b"simple_test");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Simple Test Location");
    let data_bytes = Bytes::from_slice(&env, b"simple_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &10, &3, // 3 months instead of 1
        &100, &data_hash,
    );

    // Verify it exists
    let lease_details = client.get_lease_details(&lease_id);
    assert!(lease_details.is_some());
    let lease = lease_details.unwrap();
    assert_eq!(lease.status, String::from_str(&env, "Active"));

    // Make a payment (1 of 3)
    assert!(client.process_payment(&lease_id, &lessee, &100));

    // Verify still active (not completed)
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Active"));
    assert_eq!(lease_details.payments_made, 1);

    // Terminate
    assert!(client.terminate_lease(&lease_id, &lessor));

    // Verify terminated
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Terminated"));
}

// ============ ADDITIONAL COMPREHENSIVE LEASE TESTS ============

#[test]
#[should_panic(expected = "Payment amount must be greater than 0")]
fn test_create_lease_invalid_payment_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"invalid_payment");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Invalid Payment Location");
    let data_bytes = Bytes::from_slice(&env, b"invalid_payment_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    // This should panic due to zero payment amount
    client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &12, &0, // Invalid payment amount
        &data_hash,
    );
}

#[test]
#[should_panic(expected = "Land size must be greater than 0")]
fn test_create_lease_invalid_land_size() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"invalid_size");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Invalid Size Location");
    let data_bytes = Bytes::from_slice(&env, b"invalid_size_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    // This should panic due to zero land size
    client.create_lease(
        &lessor, &lessee, &land_id, &location, &0, // Invalid land size
        &12, &1000, &data_hash,
    );
}

#[test]
fn test_multiple_lease_creation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee1, lessee2) = create_test_accounts(&env);

    client.initialize(&admin);

    // Create multiple leases
    for i in 1..=5 {
        let land_bytes = Bytes::from_slice(&env, b"multi_lease");
        let land_id = env.crypto().sha256(&land_bytes).into();
        let location = String::from_str(&env, "Multi Lease Location");
        let data_bytes = Bytes::from_slice(&env, b"multi_hash");
        let data_hash = env.crypto().sha256(&data_bytes).into();

        let lease_id = client.create_lease(
            &lessor,
            if i % 2 == 0 { &lessee1 } else { &lessee2 },
            &land_id,
            &location,
            &(100 * i),
            &(6 + i as u64),
            &(500 * i as i128),
            &data_hash,
        );

        // Verify each lease was created properly
        let lease_details = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(lease_details.status, String::from_str(&env, "Active"));
        // Note: LeaseAgreement doesn't have land_size field, land size is stored separately
        assert_eq!(lease_details.duration, 6 + i as u64);
        assert_eq!(lease_details.payment_amount, 500 * i as i128);
    }
}

#[test]
#[should_panic(expected = "Unauthorized termination attempt")]
fn test_unauthorized_lease_termination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, unauthorized) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"unauthorized_term");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Unauthorized Term Location");
    let data_bytes = Bytes::from_slice(&env, b"unauthorized_term_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash,
    );

    // This should panic - unauthorized termination
    client.terminate_lease(&lease_id, &unauthorized);
}

#[test]
fn test_lease_termination_by_lessee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"lessee_term");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Lessee Term Location");
    let data_bytes = Bytes::from_slice(&env, b"lessee_term_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash,
    );

    // Lessee should be able to terminate
    let termination_result = client.terminate_lease(&lease_id, &lessee);
    assert!(termination_result);

    // Verify termination
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Terminated"));
}

#[test]
fn test_scalability_high_volume_leases() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor1, lessor2, lessor3) = create_test_accounts(&env);
    let lessee1 = Address::generate(&env);
    let lessee2 = Address::generate(&env);

    client.initialize(&admin);

    // Create 15 leases to test scalability
    for i in 1..=15 {
        let lessor = match i % 3 {
            0 => &lessor1,
            1 => &lessor2,
            _ => &lessor3,
        };

        let lessee = if i % 2 == 0 { &lessee1 } else { &lessee2 };

        let land_bytes = Bytes::from_slice(&env, b"volume_test");
        let land_id = env.crypto().sha256(&land_bytes).into();
        let location = String::from_str(&env, "Volume Test Location");
        let data_bytes = Bytes::from_slice(&env, b"volume_hash");
        let data_hash = env.crypto().sha256(&data_bytes).into();

        let lease_id = client.create_lease(
            lessor,
            lessee,
            &land_id,
            &location,
            &(50 + (i % 10) * 10),
            &(6 + (i % 18) as u64),
            &((1000 + (i % 5) * 200) as i128),
            &data_hash,
        );

        // Verify lease creation
        let lease_details = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(lease_details.status, String::from_str(&env, "Active"));
    }
}

// ============ END-TO-END LEASE ECOSYSTEM TESTS ============

#[test]
fn test_end_to_end_lease_ecosystem() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor1, lessor2, lessor3) = create_test_accounts(&env);
    let lessee1 = Address::generate(&env);
    let lessee2 = Address::generate(&env);

    client.initialize(&admin);

    // Create ecosystem with multiple leases, payments, and potential disputes
    let mut ecosystem_leases = soroban_sdk::Vec::new(&env);

    for i in 1..=5 {
        let lessor = match i % 3 {
            0 => &lessor1,
            1 => &lessor2,
            _ => &lessor3,
        };

        let lessee = if i % 2 == 0 { &lessee1 } else { &lessee2 };

        let land_bytes = Bytes::from_slice(&env, b"ecosystem_lease");
        let land_id = env.crypto().sha256(&land_bytes).into();
        let location = String::from_str(&env, "Ecosystem Location");
        let data_bytes = Bytes::from_slice(&env, b"ecosystem_data");
        let data_hash = env.crypto().sha256(&data_bytes).into();

        let lease_id = client.create_lease(
            lessor,
            lessee,
            &land_id,
            &location,
            &(80 + i * 20),
            &4, // 4 months each
            &((600 + i * 100) as i128),
            &data_hash,
        );

        ecosystem_leases.push_back(lease_id);
    }

    // Process payments across ecosystem
    for i in 0..5 {
        let lease_id = ecosystem_leases.get(i).unwrap();
        let lease_details = client.get_lease_details(&lease_id).unwrap();
        let lessee = lease_details.lessee_id;
        let payment_amount = lease_details.payment_amount;

        // Make partial payments (2 of 4 months)
        for _month in 1..=2 {
            assert!(client.process_payment(&lease_id, &lessee, &payment_amount));
        }

        // Verify partial payment state
        let updated_lease = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(updated_lease.payments_made, 2);
        assert_eq!(updated_lease.status, String::from_str(&env, "Active"));
    }

    // Simulate various ecosystem outcomes
    // Complete first lease
    let lease_id_0 = ecosystem_leases.get(0).unwrap();
    let lease_0 = client.get_lease_details(&lease_id_0).unwrap();
    let lessee_0 = lease_0.lessee_id;
    let payment_0 = lease_0.payment_amount;

    // Complete remaining payments
    assert!(client.process_payment(&lease_id_0, &lessee_0, &payment_0));
    assert!(client.process_payment(&lease_id_0, &lessee_0, &payment_0));

    let completed_lease = client.get_lease_details(&lease_id_0).unwrap();
    assert_eq!(completed_lease.status, String::from_str(&env, "Completed"));

    // Terminate second lease due to dispute
    let lease_id_1 = ecosystem_leases.get(1).unwrap();
    let lease_1 = client.get_lease_details(&lease_id_1).unwrap();
    let lessor_1 = lease_1.lessor_id;

    assert!(client.terminate_lease(&lease_id_1, &lessor_1));

    let terminated_lease = client.get_lease_details(&lease_id_1).unwrap();
    assert_eq!(
        terminated_lease.status,
        String::from_str(&env, "Terminated")
    );
    assert_eq!(terminated_lease.payments_made, 2); // Partial payments before termination

    // Verify ecosystem integrity
    for i in 2..5 {
        let lease_id = ecosystem_leases.get(i).unwrap();
        let lease_details = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(lease_details.status, String::from_str(&env, "Active"));
        assert_eq!(lease_details.payments_made, 2);

        let payment_history = client.get_payment_history(&lease_id);
        assert_eq!(payment_history.len(), 2);
    }
}
