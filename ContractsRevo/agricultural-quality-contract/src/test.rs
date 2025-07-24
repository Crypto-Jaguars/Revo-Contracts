#![cfg(test)]

extern crate std; // Needed for vec! macro in tests sometimes

// Ensure ProductDetails is correctly imported if it's in a submodule
// use crate::product_listing::ProductDetails;
// Import necessary types from the main lib
use crate::{
    AdminError, AgricQualityContract, AgricQualityContractClient,
};

use soroban_sdk::{
    testutils::{Address as _, Ledger as _, Events as _, storage::{Persistent, Instance}},
    vec, Address, Env, IntoVal, TryFromVal, String, Symbol,
};

fn setup_test<'a>() -> (
    Env,
    Address, // Contract ID
    AgricQualityContractClient<'a>,
    Address, // Admin
    Address, // Farmer 1
    Address, // Farmer 2
) {
    let env = Env::default();
    env.mock_all_auths(); // Automatically approve all auth calls for convenience

    // Generate identities
    let admin = Address::generate(&env);
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);

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
        farmer2
    )
}


#[test]
fn test_initialize_contract() {
    let (env, _, client, admin, _, _) = setup_test();

    // Check if admin is set correctly using the non-try method
    assert_eq!(client.get_admin(), admin.clone()); // get_admin returns Result, check inner value

}



#[test]
fn test_register_product_batch_and_event() {
    let (env, contract_id, client, farmer1, _, _) = setup_test();

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
            &crate::QualityStandard::Organic,
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
    assert_eq!(cert.standard, crate::QualityStandard::Organic);
    assert_eq!(cert.holder, farmer1);
    assert_eq!(cert.status, crate::CertificationStatus::Pending);
}
