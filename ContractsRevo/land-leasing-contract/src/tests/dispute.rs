#![cfg(test)]

use super::utils::*;
use crate::*;
use soroban_sdk::{String, Bytes, testutils::Address as _};

#[test]
fn test_basic_dispute_creation_and_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"basic_dispute");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Test Farm");
    let data_bytes = Bytes::from_slice(&env, b"basic_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    let dispute_reason = String::from_str(&env, "Property damage claim");
    assert!(client.raise_dispute(&lease_id, &lessor, &dispute_reason));

    let disputed_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(disputed_lease.status, String::from_str(&env, "Disputed"));

    let resolution = String::from_str(&env, "Damage assessed - compensation required");
    assert!(client.resolve_dispute(&lease_id, &admin, &resolution));

    let resolved_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(resolved_lease.status, String::from_str(&env, "Active"));
}

#[test]
#[should_panic(expected = "Lease is not active")]
fn test_dispute_blocks_payments() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"payment_blocked");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Payment Test Farm");
    let data_bytes = Bytes::from_slice(&env, b"payment_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    assert!(client.process_payment(&lease_id, &lessee, &1000));

    let dispute_reason = String::from_str(&env, "Quality dispute");
    assert!(client.raise_dispute(&lease_id, &lessor, &dispute_reason));

    // Try to make payment while disputed - this should panic because lease is not active
    client.process_payment(&lease_id, &lessee, &1000);
}

#[test]
#[should_panic(expected = "Lease is not active")]
fn test_multiple_dispute_attempts_on_same_lease() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"multiple_disputes");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Multiple Dispute Farm");
    let data_bytes = Bytes::from_slice(&env, b"multiple_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    let dispute_reason1 = String::from_str(&env, "First dispute");
    assert!(client.raise_dispute(&lease_id, &lessor, &dispute_reason1));

    // Try to raise another dispute while first is open - this should panic because lease is not active
    let dispute_reason2 = String::from_str(&env, "Second dispute");
    client.raise_dispute(&lease_id, &lessee, &dispute_reason2);
}

#[test]
fn test_dispute_after_partial_payments() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"partial_payment_dispute");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Partial Payment Farm");
    let data_bytes = Bytes::from_slice(&env, b"partial_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &6, &500, &data_hash);

    // Make some payments first
    assert!(client.process_payment(&lease_id, &lessee, &500));
    assert!(client.process_payment(&lease_id, &lessee, &500));
    assert!(client.process_payment(&lease_id, &lessee, &500));

    let pre_dispute_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(pre_dispute_lease.payments_made, 3);
    assert_eq!(pre_dispute_lease.status, String::from_str(&env, "Active"));

    // Now raise dispute
    let dispute_reason = String::from_str(&env, "Quality degraded after payments made");
    assert!(client.raise_dispute(&lease_id, &lessee, &dispute_reason));

    let disputed_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(disputed_lease.status, String::from_str(&env, "Disputed"));
    assert_eq!(disputed_lease.payments_made, 3); // Payments should be preserved

    // Resolve and continue
    let resolution = String::from_str(&env, "Quality issue addressed");
    assert!(client.resolve_dispute(&lease_id, &admin, &resolution));

    // Continue with remaining payments
    assert!(client.process_payment(&lease_id, &lessee, &500));
    assert!(client.process_payment(&lease_id, &lessee, &500));
    assert!(client.process_payment(&lease_id, &lessee, &500));

    let final_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(final_lease.status, String::from_str(&env, "Completed"));
    assert_eq!(final_lease.payments_made, 6);
}

#[test]
fn test_dispute_resolution_with_termination_outcome() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"termination_dispute");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Termination Farm");
    let data_bytes = Bytes::from_slice(&env, b"termination_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    // Make some payments to show this isn't immediate termination
    assert!(client.process_payment(&lease_id, &lessee, &1000));

    let dispute_reason = String::from_str(&env, "Severe breach requiring termination");
    assert!(client.raise_dispute(&lease_id, &lessor, &dispute_reason));

    // Admin resolves with termination recommendation
    let resolution = String::from_str(&env, "Severe breach confirmed - termination justified");
    assert!(client.resolve_dispute(&lease_id, &admin, &resolution));

    // After resolution, terminate the lease
    assert!(client.terminate_lease(&lease_id, &lessor));

    let final_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(final_lease.status, String::from_str(&env, "Terminated"));
    assert_eq!(final_lease.payments_made, 1); // Payment history preserved
}

#[test]
fn test_lessee_initiated_dispute_vs_lessor_initiated() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor1, lessee1, _) = create_test_accounts(&env);
    let lessor2 = Address::generate(&env);
    let lessee2 = Address::generate(&env);

    client.initialize(&admin);

    // Create two identical leases
    let land_bytes1 = Bytes::from_slice(&env, b"lessor_dispute");
    let land_id1 = env.crypto().sha256(&land_bytes1).into();
    let location1 = String::from_str(&env, "Lessor Dispute Farm");
    let data_bytes1 = Bytes::from_slice(&env, b"lessor_data");
    let data_hash1 = env.crypto().sha256(&data_bytes1).into();

    let lease_id1 = client.create_lease(&lessor1, &lessee1, &land_id1, &location1, &100, &12, &1000, &data_hash1);

    let land_bytes2 = Bytes::from_slice(&env, b"lessee_dispute");
    let land_id2 = env.crypto().sha256(&land_bytes2).into();
    let location2 = String::from_str(&env, "Lessee Dispute Farm");
    let data_bytes2 = Bytes::from_slice(&env, b"lessee_data");
    let data_hash2 = env.crypto().sha256(&data_bytes2).into();

    let lease_id2 = client.create_lease(&lessor2, &lessee2, &land_id2, &location2, &100, &12, &1000, &data_hash2);

    // Lessor-initiated dispute
    let lessor_dispute_reason = String::from_str(&env, "Lessee breach of contract");
    assert!(client.raise_dispute(&lease_id1, &lessor1, &lessor_dispute_reason));

    // Lessee-initiated dispute
    let lessee_dispute_reason = String::from_str(&env, "Lessor breach of obligations");
    assert!(client.raise_dispute(&lease_id2, &lessee2, &lessee_dispute_reason));

    // Both should result in disputed status
    let disputed_lease1 = client.get_lease_details(&lease_id1).unwrap();
    let disputed_lease2 = client.get_lease_details(&lease_id2).unwrap();

    assert_eq!(disputed_lease1.status, String::from_str(&env, "Disputed"));
    assert_eq!(disputed_lease2.status, String::from_str(&env, "Disputed"));

    // Different resolutions
    let lessor_resolution = String::from_str(&env, "Lessee must remedy breach");
    let lessee_resolution = String::from_str(&env, "Lessor must fulfill obligations");

    assert!(client.resolve_dispute(&lease_id1, &admin, &lessor_resolution));
    assert!(client.resolve_dispute(&lease_id2, &admin, &lessee_resolution));
}

#[test]
fn test_dispute_state_persistence_across_operations() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"persistence_test");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Persistence Farm");
    let data_bytes = Bytes::from_slice(&env, b"persistence_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    // Initial state
    let initial_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(initial_lease.status, String::from_str(&env, "Active"));

    // Raise dispute
    let dispute_reason = String::from_str(&env, "Testing state persistence");
    assert!(client.raise_dispute(&lease_id, &lessor, &dispute_reason));

    // Check state changed
    let disputed_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(disputed_lease.status, String::from_str(&env, "Disputed"));

    // Try other operations to ensure dispute state persists
    let _payment_history_during_dispute = client.get_payment_history(&lease_id);
    let _land_details = client.get_land_details(&land_id);
    let _user_leases = client.get_user_leases(&lessee);

    // State should still be disputed after other operations
    let still_disputed = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(still_disputed.status, String::from_str(&env, "Disputed"));

    // Resolve dispute
    let resolution = String::from_str(&env, "State persistence confirmed");
    assert!(client.resolve_dispute(&lease_id, &admin, &resolution));

    // Final state check
    let resolved_lease = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(resolved_lease.status, String::from_str(&env, "Active"));
}

#[test]
#[should_panic(expected = "Only lease parties can raise disputes")]
fn test_unauthorized_dispute_creation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, outsider) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"unauthorized_test");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Unauthorized Farm");
    let data_bytes = Bytes::from_slice(&env, b"unauthorized_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    let dispute_reason = String::from_str(&env, "Unauthorized interference");
    client.raise_dispute(&lease_id, &outsider, &dispute_reason);
}

#[test]
#[should_panic(expected = "Unauthorized resolver")]
fn test_unauthorized_dispute_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, unauthorized) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"unauthorized_resolution");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Resolution Farm");
    let data_bytes = Bytes::from_slice(&env, b"resolution_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    let dispute_reason = String::from_str(&env, "Valid dispute");
    assert!(client.raise_dispute(&lease_id, &lessor, &dispute_reason));

    let resolution = String::from_str(&env, "Unauthorized resolution");
    client.resolve_dispute(&lease_id, &unauthorized, &resolution);
}

#[test]
#[should_panic(expected = "Dispute reason cannot be empty")]
fn test_empty_dispute_reason() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"empty_reason");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Empty Reason Farm");
    let data_bytes = Bytes::from_slice(&env, b"empty_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    let empty_reason = String::from_str(&env, "");
    client.raise_dispute(&lease_id, &lessor, &empty_reason);
}

#[test]
#[should_panic(expected = "Lease is not active")]
fn test_dispute_on_terminated_lease() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);

    let (admin, lessor, lessee, _) = create_test_accounts(&env);
    client.initialize(&admin);

    let land_bytes = Bytes::from_slice(&env, b"terminated_dispute");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Terminated Farm");
    let data_bytes = Bytes::from_slice(&env, b"terminated_data");
    let data_hash = env.crypto().sha256(&data_bytes).into();

    let lease_id = client.create_lease(&lessor, &lessee, &land_id, &location, &100, &12, &1000, &data_hash);

    assert!(client.terminate_lease(&lease_id, &lessor));

    let dispute_reason = String::from_str(&env, "Post-termination dispute");
    client.raise_dispute(&lease_id, &lessor, &dispute_reason);
}