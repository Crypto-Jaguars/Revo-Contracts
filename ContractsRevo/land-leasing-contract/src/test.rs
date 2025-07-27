#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, Address, Env, String, Bytes,
};

fn create_test_contract(env: &Env) -> Address {
    env.register(LandLeasingContract, ())
}

fn create_test_accounts(env: &Env) -> (Address, Address, Address, Address) {
    (
        Address::generate(env),
        Address::generate(env),
        Address::generate(env),
        Address::generate(env),
    )
}

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let contract_id = create_test_contract(&env);
    let client = LandLeasingContractClient::new(&env, &contract_id);
    
    let (admin, _, _, _) = create_test_accounts(&env);
    
    client.initialize(&admin);
    
    // Verify admin is set - use as_contract to access storage
    let is_admin_result = env.as_contract(&contract_id, || {
        utils::is_admin(&env, &admin)
    });
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
    
    // Fix bytes conversion - use proper string encoding
    let land_bytes = Bytes::from_slice(&env, b"test_land_1");
    let land_id = env.crypto().sha256(&land_bytes).into();
    let location = String::from_str(&env, "Test Farm Location");
    let data_bytes = Bytes::from_slice(&env, b"land_data_hash");
    let data_hash = env.crypto().sha256(&data_bytes).into();
    
    let lease_id = client.create_lease(
        &lessor,
        &lessee,
        &land_id,
        &location,
        &100, // 100 hectares
        &12, // 12 months
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
        &lessor,
        &lessee,
        &land_id,
        &location,
        &50,
        &6,
        &500,
        &data_hash,
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
        &lessor,
        &lessee,
        &land_id,
        &location,
        &75,
        &8,
        &800,
        &data_hash,
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
        &lessor,
        &lessee,
        &land_id,
        &location,
        &100,
        &0, // Invalid duration
        &1000,
        &data_hash,
    );
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
        &lessor,
        &lessee,
        &land_id,
        &location,
        &50,
        &6,
        &500,
        &data_hash,
    );
    
    // This should panic - other_user is not the lessee
    client.process_payment(&lease_id, &other_user, &500);
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
        &lessor,
        &lessee,
        &land_id,
        &location,
        &10,
        &3, // 3 months instead of 1
        &100,
        &data_hash,
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
        &lessor,
        &lessee,
        &land_id,
        &location,
        &10,
        &1, // 1 month - will complete after 1 payment
        &100,
        &data_hash,
    );
    
    // Make the payment (completes the lease)
    assert!(client.process_payment(&lease_id, &lessee, &100));
    
    // Verify lease is completed
    let lease_details = client.get_lease_details(&lease_id).unwrap();
    assert_eq!(lease_details.status, String::from_str(&env, "Completed"));
    assert_eq!(lease_details.payments_made, 1);
    assert_eq!(lease_details.total_payments_required, 1);
}
