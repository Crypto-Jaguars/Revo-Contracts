#![cfg(test)]

extern crate std; // Needed for vec! macro in tests sometimes

// use core::result;

// Ensure ProductDetails is correctly imported if it's in a submodule
// use crate::product_listing::ProductDetails;
// Import necessary types from the main lib
use crate::{
    AdminError, AgricQualityContract, AgricQualityContractClient, AgricQualityError,
};
use crate::QualityStandard;
use crate::CertificationStatus;
use crate::DataKey;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _, Events as _, storage::{Persistent, Instance}},
    log, vec, Address, Env, IntoVal, TryFromVal, String, Symbol,
};

// mod test_utils; // Module for helper functions

// use test_utils::*; // Import functions from test_utils

fn setup_test<'a>() -> (
    Env,
    Address, // Contract ID
    AgricQualityContractClient<'a>,
    Address, // Admin
    Address, // Farmer 1
    Address, // Inspector
    Address, // Authority
) {
    let env = Env::default();
    env.mock_all_auths(); // Automatically approve all auth calls for convenience

    // Generate identities
    let admin = Address::generate(&env);
    let farmer1 = Address::generate(&env);
    let inspector = Address::generate(&env);
    let authority = Address::generate(&env);


    // Register the contract
    // Use register_contract for contracts, not register() which is for custom types
    let contract_id = env.register(AgricQualityContract, ());
    let client = AgricQualityContractClient::new(&env, &contract_id);

    // Initialize the contract
    client.initialize(&admin);

    (
        env,
        contract_id, // Return the contract Address (ID)
        client,
        admin,
        farmer1,
        inspector,
        authority
    )
}

// Test initialization
#[test]
#[should_panic]
fn test_initialize_contract() {
    let (env, _, client, admin, _, _, _) = setup_test();

    // Check if admin is set correctly using the non-try method
    assert_eq!(client.get_admin(), admin.clone());

    // Attempt to initialize again (should fail)
    let result = client.initialize(&admin);
}

// Test register product batch and event
#[test]
fn test_register_product_batch_and_event() {
    let (env, _, client, _, farmer1, _, _) = setup_test();

    // Prepare metadata
    let crop_type = String::from_str(&env, "Coffee");
    let harvest_date = String::from_str(&env, "23:07:2025");
    let origin = String::from_str(&env, "Tarrazu");
    let metadata = vec![
        &env,
        String::from_str(&env, "crop_type"),
        crop_type,
        String::from_str(&env, "harvest_date"),
        harvest_date,
        String::from_str(&env, "origin"),
        origin,
        String::from_str(&env, "producer"),
        farmer1.to_string(),
    ];

    // Register product batch
    let cert_id = client
        .submit_for_certification(
            &farmer1,
            &QualityStandard::Organic,
            &metadata,
        );

    // Check if the certification ID is valid
    assert!(cert_id.len() > 0, "Certification ID should not be empty.");

    // Check event emission
    let events = env.events().all().last().unwrap();
    let (actual_contract_id_val, topics_val, data_val) = events;

    let topic_symbol = Symbol::try_from_val(&env, &topics_val.get(0).unwrap().clone()).unwrap();
    assert_eq!(
        topic_symbol,
        Symbol::new(&env, "certification_submitted"),
        "Expected 'certification_submitted' as the first topic."
    );

    // Check certification data exists
    let history = client.get_certification_history(&farmer1);
    std::println!("Certification history: {:?}", history);

    assert_eq!(history.len(), 1);
    let cert = &history.get(0).unwrap();
    assert_eq!(cert.standard, QualityStandard::Organic);
    assert_eq!(cert.holder, farmer1);
    assert_eq!(cert.status, CertificationStatus::Pending);
}


// Test duplicate registration is rejected
#[test]
#[should_panic]
fn test_duplicate_registration_rejected() {
    let (env, _, client, _, farmer1, _, _) = setup_test();

    let metadata = vec![&env, String::from_str(&env, "batch"), String::from_str(&env, "A")];

    // First registration
    client.submit_for_certification(
        &farmer1,
        &QualityStandard::GlobalGAP,
        &metadata.clone(),
    );

    // Duplicate registration attempt (should fail)
    client.submit_for_certification(
        &farmer1,
        &QualityStandard::GlobalGAP,
        &metadata,
    );
}

// Test incomplete metadata is rejected
#[test]
#[should_panic]
fn test_incomplete_metadata_rejected() {
    let (env, _, client, _, farmer1, _, _) = setup_test();
    // Missing required fields (simulate with empty metadata)
    let metadata = vec![&env];
    client.submit_for_certification(
        &farmer1,
        &QualityStandard::GlobalGAP,
        &metadata.clone(),
    );
}


// Test metric registration and retrieval
#[test]
fn test_register_and_get_metric() {
    let (env, _, client, admin, _, _, authority) = setup_test();


    // client.add_authority(&admin, &authority);
    let result = client.add_authority(&admin, &authority);
    assert!(result == authority, "authority not added.");


    // Register a metric
    let metric_name = Symbol::new(&env, "moisture");
    client.register_metric(&authority, &QualityStandard::Organic, &metric_name, &90, &10);
    // assert!(result);

    // Get the metrics for the standard
    let metrics = client.get_standard_metrics(&QualityStandard::Organic);
    assert_eq!(metrics.len(), 1); // Assuming no other metrics are registered
    assert_eq!(metrics.get(0).unwrap().name, metric_name);
    assert_eq!(metrics.get(0).unwrap().min_score, 90);
    assert_eq!(metrics.get(0).unwrap().weight, 10);

}


// Test metric update
#[test]
fn test_update_metric() {
    let (env, _, client, admin, _, _, authority) = setup_test();

    let result = client.add_authority(&admin, &authority);
    assert!(result == authority, "authority not added.");

    // Register a metric first
    let metric_name = Symbol::new(&env, "moisture");
    client.register_metric(&authority, &QualityStandard::Organic, &metric_name, &90, &10);

    // Update the metric
    client.update_metric(&authority, &QualityStandard::Organic, &metric_name, &95, &15);

    // Get the metrics and check if updated
    let metrics = client.get_standard_metrics(&QualityStandard::Organic);
    assert_eq!(metrics.get(0).unwrap().min_score, 95);
    assert_eq!(metrics.get(0).unwrap().weight, 15);
}

// // Test access control for metric registration
#[test]
#[should_panic]
fn test_register_metric_unauthorized() {
    let (env, _, client, _, farmer1, _, _) = setup_test();

    // Attempt to register a metric with a non-admin address
    let metric_name = Symbol::new(&env, "size");
    client.register_metric(&farmer1, &QualityStandard::Organic, &metric_name, &80, &5);
    
}


// Test get certification history
#[test]
fn test_get_certification_history() {
    let (env, _, client, _, farmer1, _, _) = setup_test();

    // Register a product batch
    let metadata = vec![&env, String::from_str(&env, "batch"), String::from_str(&env, "D")];
    client.submit_for_certification(
        &farmer1,
        &QualityStandard::GlobalGAP,
        &metadata,
    );

    // Get certification history
    let history = client.get_certification_history(&farmer1);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().holder, farmer1);
}



