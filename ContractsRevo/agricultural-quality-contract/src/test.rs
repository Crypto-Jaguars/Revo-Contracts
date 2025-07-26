#![cfg(test)]

use crate::CertificationStatus;
use crate::DisputeStatus;
use crate::QualityStandard;
use crate::ResolutionOutcome;
use crate::{AgricQualityContract, AgricQualityContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _},
    vec, Address, Bytes, BytesN, Env, String, Symbol, TryFromVal,
};

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
        authority,
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
    client.initialize(&admin);
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
    let cert_id = client.submit_for_certification(&farmer1, &QualityStandard::Organic, &metadata);

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

    let metadata = vec![
        &env,
        String::from_str(&env, "batch"),
        String::from_str(&env, "A"),
    ];

    // First registration
    client.submit_for_certification(&farmer1, &QualityStandard::GlobalGAP, &metadata);

    // Duplicate registration attempt (should fail)
    client.submit_for_certification(&farmer1, &QualityStandard::GlobalGAP, &metadata);
}

// Test incomplete metadata is rejected
#[test]
#[should_panic]
fn test_incomplete_metadata_rejected() {
    let (env, _, client, _, farmer1, _, _) = setup_test();
    // Missing required fields (simulate with empty metadata)
    let metadata = vec![&env];
    client.submit_for_certification(&farmer1, &QualityStandard::GlobalGAP, &metadata.clone());
}

// Test metric registration and retrieval
#[test]
fn test_register_and_get_metric() {
    let (env, _, client, admin, _, _, authority) = setup_test();

    let result = client.add_authority(&admin, &authority);
    assert_eq!(result, authority, "Failed to add authority");

    // Register a metric
    let metric_name = Symbol::new(&env, "moisture");
    client.register_metric(
        &authority,
        &QualityStandard::Organic,
        &metric_name,
        &90,
        &10,
    );
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
    assert_eq!(result, authority, "Failed to add authority");

    // Register a metric first
    let metric_name = Symbol::new(&env, "moisture");
    client.register_metric(
        &authority,
        &QualityStandard::Organic,
        &metric_name,
        &90,
        &10,
    );

    // Update the metric
    client.update_metric(
        &authority,
        &QualityStandard::Organic,
        &metric_name,
        &95,
        &15,
    );

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
    let metadata = vec![
        &env,
        String::from_str(&env, "batch"),
        String::from_str(&env, "D"),
    ];
    client.submit_for_certification(&farmer1, &QualityStandard::GlobalGAP, &metadata);

    // Get certification history
    let history = client.get_certification_history(&farmer1);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().holder, farmer1);
}

// Test record inspection
#[test]
fn test_record_inspection() {
    let (env, _, client, admin, farmer1, inspector, _) = setup_test();

    let result = client.add_inspector(&admin, &inspector);
    assert_eq!(result, inspector, "Failed to add inspector");

    // Register a product batch first
    let metadata = vec![
        &env,
        String::from_str(&env, "batch"),
        String::from_str(&env, "B"),
    ];
    let cert_id = client.submit_for_certification(&farmer1, &QualityStandard::GlobalGAP, &metadata);

    // Record an inspection
    let metrics = vec![&env, (Symbol::short("moisture"), 92_u32)];
    let findings = vec![&env, String::from_str(&env, "Good moisture level")];
    let recommendations = vec![&env, String::from_str(&env, "None needed")];

    client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);
}

// Test process certification
#[test]
fn test_process_certification() {
    let (env, _, client, admin, farmer1, inspector, issuer) = setup_test();

    client.add_authority(&admin, &issuer); // Make sure issuer is added as an authority
    client.add_inspector(&admin, &inspector); // Make sure inspector is added as an inspector

    // 2. Submit for Certification
    let metadata = vec![
        &env,
        String::from_str(&env, "batch"),
        String::from_str(&env, "B"),
    ];
    let cert_id = client.submit_for_certification(&farmer1, &QualityStandard::GlobalGAP, &metadata);

    // 3. Record an Inspection for that certification
    let metrics = vec![&env, (Symbol::short("moisture"), 92_u32)];
    let findings = vec![&env, String::from_str(&env, "Good moisture level")];
    let recommendations = vec![&env, String::from_str(&env, "None needed")];
    client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);

    // 4. Process the Certification
    let approved = true;
    let validity_period = 31536000; // 1 year in seconds
    client.process_certification(&issuer, &cert_id, &approved, &validity_period);
}

// Test dispute filing
#[test]
fn test_file_dispute() {
    let (env, admin, client, authority, farmer1, inspector, _) = setup_test();

    // Add farmer1 as an authority since process_certification requires an issuer which is an authority
    client.add_authority(&admin, &inspector);

    client.add_inspector(&admin, &inspector);

    // Register a product batch for certification
    let conditions = vec![&env, String::from_str(&env, "organic_soil_used")];
    let cert_id = client.submit_for_certification(&farmer1, &QualityStandard::Organic, &conditions);

    let metrics = vec![
        &env,
        (Symbol::new(&env, "score_a"), 90),
        (Symbol::new(&env, "score_b"), 85),
    ];
    let findings = vec![
        &env,
        String::from_str(&env, "Soil sample good"),
        String::from_str(&env, "Pesticide test negative"),
    ];
    let recommendations = vec![&env, String::from_str(&env, "Continue monitoring")];

    client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);

    // Process the certification to make it valid
    client.process_certification(&inspector, &cert_id, &true, &1000);

    // File a dispute with valid evidence
    let description = String::from_str(&env, "The organic produce contained pesticides.");
    let evidence_hash_1 = env.crypto().sha256(&Bytes::from_array(&env, &[1; 32]));
    let evidence_hash_2 = env.crypto().sha256(&Bytes::from_array(&env, &[2; 32]));
    let evidence = vec![&env, evidence_hash_1.into(), evidence_hash_2.into()];

    let dispute_id = client.file_dispute(&farmer1, &cert_id, &description, &evidence);

    // Verify the dispute details
    let stored_dispute = client.get_dispute_details(&dispute_id);

    assert_eq!(stored_dispute.complainant, farmer1);
    assert_eq!(stored_dispute.certification, cert_id);
    assert_eq!(stored_dispute.description, description);
    assert_eq!(stored_dispute.evidence.len(), 2);
    assert_eq!(stored_dispute.status, DisputeStatus::Filed);
    assert_eq!(stored_dispute.resolution, ResolutionOutcome::Pending);
}

#[test]
#[should_panic]
fn test_file_dispute_bad() {
    let (env, admin, client, authority, farmer1, inspector, _) = setup_test();

    // Add farmer1 as an authority since process_certification requires an issuer which is an authority
    client.add_authority(&admin, &inspector);

    client.add_inspector(&admin, &inspector);

    // Register a product batch for certification
    let conditions = vec![&env, String::from_str(&env, "organic_soil_used")];
    let cert_id = client.submit_for_certification(&farmer1, &QualityStandard::Organic, &conditions);

    let metrics = vec![
        &env,
        (Symbol::new(&env, "score_a"), 90),
        (Symbol::new(&env, "score_b"), 85),
    ];
    let findings = vec![
        &env,
        String::from_str(&env, "Soil sample good"),
        String::from_str(&env, "Pesticide test negative"),
    ];
    let recommendations = vec![&env, String::from_str(&env, "Continue monitoring")];

    client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);

    // Process the certification to make it valid
    client.process_certification(&inspector, &cert_id, &true, &1000);

    // Test filing a dispute with no evidence (should fail)
    let description = String::from_str(&env, "The organic produce contained pesticides.");
    let empty_evidence = vec![&env];

    client.file_dispute(&farmer1, &cert_id, &description, &empty_evidence);

    let evidence_hash_1 = env.crypto().sha256(&Bytes::from_array(&env, &[1; 32]));
    let evidence_hash_2 = env.crypto().sha256(&Bytes::from_array(&env, &[2; 32]));
    let evidence = vec![&env, evidence_hash_1.into(), evidence_hash_2.into()];
    // Test filing a dispute with an invalid certification ID (should fail)
    let invalid_cert_id = BytesN::from_array(&env, &[0; 32]);
    client.file_dispute(&farmer1, &invalid_cert_id, &description, &evidence);
}
