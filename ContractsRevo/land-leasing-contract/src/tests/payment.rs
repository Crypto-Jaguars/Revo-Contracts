#![cfg(test)]

use super::utils::*;
use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, String};

#[test]
fn test_process_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"test_land_2");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Test Farm Location 2");
    let data_bytes = Bytes::from_slice(&env, b"land_data_hash_2");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &50, &6, &500, &data_hash,
    );

    // Process payment
    let payment_result = client.process_payment(&lease_id, &lessee, &500);
    assert!(payment_result);

    // Check payment history
    let payment_history = client.get_payment_history(&lease_id);
    assert_eq!(payment_history.len(), 1);

    let payment = payment_history.get(0).unwrap();
    assert_eq!(payment.payer, lessee);
    assert_eq!(payment.amount, 500);
    assert_eq!(payment.lease_id, lease_id);
}

#[test]
#[should_panic(expected = "Only lessee can make payments")]
fn test_payment_by_wrong_user() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, other_user) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"test_land_payment");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Payment Test Location");
    let data_bytes = Bytes::from_slice(&env, b"payment_data_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &50, &6, &500, &data_hash,
    );

    // This should panic - other_user is not the lessee
    client.process_payment(&lease_id, &other_user, &500);
}

#[test]
fn test_lease_completion() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    // Initialize
    client.initialize(&admin);

    // Create a 1-month lease that will auto-complete
    let land_bytes = Bytes::from_slice(&env, b"completion_test");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Completion Test Location");
    let data_bytes = Bytes::from_slice(&env, b"completion_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &10,
        &1, // 1 month - will complete after 1 payment
        &100, &data_hash,
    );

    // Make the payment (completes the lease)
    assert!(client.process_payment(&lease_id, &lessee, &100));

    // Verify lease is completed
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Completed"));
    assert_eq!(lease_details.payments_made, 1);
    assert_eq!(lease_details.total_payments_required, 1);
}

// ============ RECURRING PAYMENTS AND SCHEDULE ADHERENCE ============

#[test]
fn test_recurring_payments_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"recurring_test");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Recurring Payment Location");
    let data_bytes = Bytes::from_slice(&env, b"recurring_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &5, // 5 months
        &200, &data_hash,
    );

    // Make payments over multiple months
    for i in 1..=5 {
        assert!(client.process_payment(&lease_id, &lessee, &200));

        let lease_details = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(lease_details.payments_made, i);

        if i < 5 {
            assert_eq!(lease_details.status, String::from_str(&env, "Active"));
        } else {
            assert_eq!(lease_details.status, String::from_str(&env, "Completed"));
        }
    }
}

#[test]
#[should_panic(expected = "Insufficient payment amount")]
fn test_incorrect_payment_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"incorrect_amount");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Incorrect Amount Location");
    let data_bytes = Bytes::from_slice(&env, b"incorrect_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &12, &1000, // Expected payment is 1000
        &data_hash,
    );

    // This should panic - incorrect payment amount
    client.process_payment(&lease_id, &lessee, &800); // Wrong amount
}

#[test]
#[should_panic(expected = "Insufficient payment amount")]
fn test_payment_insufficient_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"insufficient_funds");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Insufficient Funds Location");
    let data_bytes = Bytes::from_slice(&env, b"insufficient_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash,
    );

    // This should panic - insufficient payment amount (paying less than required)
    client.process_payment(&lease_id, &lessee, &500); // Less than required 1000
}

#[test]
fn test_payment_schedule_adherence() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"schedule_test");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Schedule Test Location");
    let data_bytes = Bytes::from_slice(&env, b"schedule_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &150, &6, // 6 months
        &300, &data_hash,
    );

    // Make payments according to schedule
    for i in 1..=6 {
        assert!(client.process_payment(&lease_id, &lessee, &300));

        let lease_details = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(lease_details.payments_made, i);

        let payment_history = client.get_payment_history(&lease_id);
        assert_eq!(payment_history.len(), i);

        // Verify payment details
        let latest_payment = payment_history.get((i - 1) as u32).unwrap();
        assert_eq!(latest_payment.payer, lessee);
        assert_eq!(latest_payment.amount, 300);
    }

    // Verify final completion
    let final_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(final_lease.status, String::from_str(&env, "Completed"));
    assert_eq!(final_lease.payments_made, 6);
    assert_eq!(final_lease.total_payments_required, 6);
}

#[test]
fn test_early_lease_termination_with_payments() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"early_termination");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Early Termination Location");
    let data_bytes = Bytes::from_slice(&env, b"early_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &10, // 10 months
        &500, &data_hash,
    );

    // Make partial payments (3 of 10)
    for i in 1..=3 {
        assert!(client.process_payment(&lease_id, &lessee, &500));
        let lease_details = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(lease_details.payments_made, i);
        assert_eq!(lease_details.status, String::from_str(&env, "Active"));
    }

    // Early termination by lessor
    assert!(client.terminate_lease(&lease_id, &lessor));

    // Verify termination with partial payments
    let terminated_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(
        terminated_lease.status,
        String::from_str(&env, "Terminated")
    );
    assert_eq!(terminated_lease.payments_made, 3);

    let payment_history = client.get_payment_history(&lease_id);
    assert_eq!(payment_history.len(), 3);
}

// ============ SCALABILITY AND INTEGRATION TESTS ============

#[test]
fn test_multiple_lease_payment_processing() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor1, lessor2, lessor3) = create_test_accounts(&env);
    let lessee1 = Address::generate(&env);
    let lessee2 = Address::generate(&env);

    client.initialize(&admin);

    let mut lease_ids = soroban_sdk::Vec::new(&env);

    // Create 10 different leases
    for i in 1..=10 {
        let lessor = match i % 3 {
            0 => &lessor1,
            1 => &lessor2,
            _ => &lessor3,
        };

        let lessee = if i % 2 == 0 { &lessee1 } else { &lessee2 };

        let land_bytes = Bytes::from_slice(&env, b"multi_lease");
        let land_id = env.crypto().sha256(&land_bytes).into();
        let location = String::from_str(&env, "Multi Lease Location");
        let data_bytes = Bytes::from_slice(&env, b"multi_hash");
        let data_hash = env.crypto().sha256(&data_bytes).into();

        let lease_id = client.create_lease(
            lessor,
            lessee,
            &land_id,
            &location,
            &(50 + (i % 5) * 10),
            &3, // 3 months each
            &((400 + (i % 3) * 100) as i128),
            &data_hash,
        );

        lease_ids.push_back(lease_id);
    }

    // Process payments for all leases
    for i in 0..10 {
        let lease_id = lease_ids.get(i).unwrap();
        let lease_details = client.get_lease_details(&lease_id).unwrap();
        let payment_amount = lease_details.payment_amount;
        let lessee = lease_details.lessee_id;

        // Make all 3 payments to complete each lease
        for month in 1..=3 {
            assert!(client.process_payment(&lease_id, &lessee, &payment_amount));

            let updated_lease = client.get_lease_details(&lease_id).unwrap();
            assert_eq!(updated_lease.payments_made, month);
        }

        // Verify completion
        let final_details = client.get_lease_details(&lease_id).unwrap();
        assert_eq!(final_details.status, String::from_str(&env, "Completed"));
        assert_eq!(final_details.payments_made, 3);
    }
}

// ============ COMMODITY TOKEN INTEGRATION TESTS ============

#[test]
fn test_commodity_token_integration_placeholder() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"token_integration");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Token Integration Location");
    let data_bytes = Bytes::from_slice(&env, b"token_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash,
    );

    // Test foundation exists for commodity token integration
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Active"));

    // Placeholder for commodity token integration testing
    // In real implementation, would integrate with commodity-token-contract
    // and test tokenized payments, but for now verify payment system works
    assert!(client.process_payment(&lease_id, &lessee, &1000));

    let payment_history = client.get_payment_history(&lease_id);
    assert_eq!(payment_history.len(), 1);
}

#[test]
fn test_tokenized_payment_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);

    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"tokenized_payment");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Tokenized Payment Location");
    let data_bytes = Bytes::from_slice(&env, b"tokenized_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(
        &lessor, &lessee, &land_id, &location, &150, &8, &750, &data_hash,
    );

    // Test foundation for tokenized payments
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Active"));
    assert_eq!(lease_details.payment_amount, 750);

    // Make regular payment for now (would be tokenized in real implementation)
    assert!(client.process_payment(&lease_id, &lessee, &750));

    let payment_history = client.get_payment_history(&lease_id);
    assert_eq!(payment_history.len(), 1);
}
