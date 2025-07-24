#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl, log, symbol_short,
    testutils::{Address as _, Ledger},
    xdr::ToXdr,
    Address, BytesN, Env, Map, String, Symbol,
};

// Test Constants
const TEST_EXPIRATION_DATE: u64 = 2000000000; // Far future timestamp

/// Create a test environment with the necessary contracts and clients.
fn create_test_contracts(
    env: &Env,
) -> (
    Address,
    Address,
    SupplyChainTrackingContractClient,
    MockCertificateManagementClient,
) {
    let supply_chain_id = env.register(SupplyChainTrackingContract, ());
    let cert_mgmt_id = env.register(MockCertificateManagement, ());
    let supply_chain_client = SupplyChainTrackingContractClient::new(&env, &supply_chain_id);
    let cert_mgmt_client = MockCertificateManagementClient::new(&env, &cert_mgmt_id);

    (
        supply_chain_id,
        cert_mgmt_id,
        supply_chain_client,
        cert_mgmt_client,
    )
}

/// Setup test environment with initialized contracts
fn setup_test_environment(
    env: &Env,
) -> (
    Address,
    Address,
    Address,
    Address,
    SupplyChainTrackingContractClient,
    MockCertificateManagementClient,
) {
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let handler = Address::generate(&env);
    let authority = Address::generate(&env);

    let (_, cert_mgmt_id, supply_chain_client, cert_mgmt_client) = create_test_contracts(&env);

    // Initialize supply chain contract with certificate management contract
    supply_chain_client.initialize(&admin, &cert_mgmt_id);

    (
        admin,
        farmer,
        handler,
        authority,
        supply_chain_client,
        cert_mgmt_client,
    )
}

/// Create test product registration data
fn create_test_product_data(env: &Env, prefix: &str) -> (String, String, String, BytesN<32>) {
    // Helper to concatenate prefix and suffix as bytes, then convert to &str
    let concat = |prefix: &str, suffix: &str| -> String {
        let prefix_bytes = prefix.as_bytes();
        let suffix_bytes = suffix.as_bytes();
        let len = prefix_bytes.len() + suffix_bytes.len();

        let mut buf = [0u8; 64];
        assert!(len <= buf.len());
        buf[..prefix_bytes.len()].copy_from_slice(prefix_bytes);
        buf[prefix_bytes.len()..len].copy_from_slice(suffix_bytes);

        let s = core::str::from_utf8(&buf[..len]).unwrap();
        String::from_str(env, s)
    };

    let product_type = concat(prefix, "_Organic_Tomatoes");
    let batch_number = concat(prefix, "_BATCH001");
    let origin_location = concat(prefix, "_Farm_Location");
    let metadata_hash = BytesN::from_array(env, &[1u8; 32]);
    (product_type, batch_number, origin_location, metadata_hash)
}

/// Create test certificate data
fn create_test_certificate_data(env: &Env) -> (CertificateId, CertStatus) {
    let certificate_id = CertificateId::Some(BytesN::from_array(&env, &[2u8; 32]));
    let status = CertStatus::Valid;
    (certificate_id, status)
}

/// Create test certificate validation hash
fn create_test_validation_hash(
    env: &Env,
    farmer: &Address,
    product_id: &BytesN<32>,
    stage_id: u8,
) -> BytesN<32> {
    let mut combined_data = soroban_sdk::Bytes::new(&env);
    combined_data.append(&soroban_sdk::Bytes::from_array(
        &env,
        &product_id.to_array(),
    ));
    combined_data.append(&farmer.clone().to_xdr(&env));
    combined_data.append(&soroban_sdk::Bytes::from_array(&env, &[stage_id; 32])); // stage hash
    combined_data.append(&soroban_sdk::Bytes::from_array(
        &env,
        &env.ledger().timestamp().to_be_bytes(),
    ));
    env.crypto().sha256(&combined_data).into()
}

/// Setup certificate in mock contract
fn setup_mock_certificate(
    cert_client: &MockCertificateManagementClient,
    owner: &Address,
    authority: &Address,
    certificate_id: &BytesN<32>,
    status: CertStatus,
    verification_hash: BytesN<32>,
) {
    let env = &cert_client.env;
    let cert_id = utils::convert_bytes_to_u32(env, certificate_id);

    cert_client.set_cert_status(&owner, &cert_id, &status);
    cert_client.set_cert_verification_hash(&owner, &cert_id, &verification_hash);

    let certification = Certification::new(
        cert_id,
        symbol_short!("ORGANIC"),
        authority.clone(),
        1000000,
        TEST_EXPIRATION_DATE,
        verification_hash,
    );
    cert_client.set_certification(&owner, &cert_id, &certification);
}

// =====================================================================================
// INITIALIZATION TESTS
// =====================================================================================

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let cert_mgmt_contract = Address::generate(&env);

    let supply_chain_id = env.register(SupplyChainTrackingContract, ());
    let supply_chain_client = SupplyChainTrackingContractClient::new(&env, &supply_chain_id);

    // Initialize contract
    supply_chain_client.initialize(&admin, &cert_mgmt_contract.clone());

    // Verify admin is set
    let stored_admin = supply_chain_client.get_admin();
    assert_eq!(stored_admin, admin, "Admin should be set correctly");

    // Verify certificate management contract is set
    let stored_cert_contract = supply_chain_client.get_cert_mgmt_contract();
    assert_eq!(
        stored_cert_contract, cert_mgmt_contract,
        "Certificate management contract should be set"
    );

    // Verify initialization event
    // let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
    // assert_eq!(events.len(), 1, "Expected one initialization event");
    // let expected_event_topic = (Symbol::new(&env, "contract_initialized"), admin.clone());
    // assert!(
    //     events
    //         .iter()
    //         .any(|(_, topic, _)| topic == expected_event_topic.into_val(&env)),
    //     "Should emit contract_initialized event"
    // );
}

#[test]
fn test_initialize_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let cert_mgmt_contract = Address::generate(&env);

    let supply_chain_id = env.register(SupplyChainTrackingContract, ());
    let supply_chain_client = SupplyChainTrackingContractClient::new(&env, &supply_chain_id);

    // Initialize first time
    supply_chain_client.initialize(&admin, &cert_mgmt_contract.clone());

    // Try to initialize again - should fail
    let result = supply_chain_client.try_initialize(&admin, &cert_mgmt_contract);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::AlreadyInitialized)),
        "Should fail on double initialization"
    );
}

#[test]
fn test_set_cert_mgmt_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let initial_cert_contract = Address::generate(&env);
    let new_cert_contract = Address::generate(&env);

    let supply_chain_id = env.register(SupplyChainTrackingContract, ());
    let supply_chain_client = SupplyChainTrackingContractClient::new(&env, &supply_chain_id);

    // Initialize with initial certificate contract
    supply_chain_client.initialize(&admin, &initial_cert_contract);

    // Update certificate management contract
    supply_chain_client.set_cert_mgmt_contract(&admin, &new_cert_contract);

    // Verify update
    let stored_cert_contract = supply_chain_client.get_cert_mgmt_contract();
    assert_eq!(
        stored_cert_contract, new_cert_contract,
        "Certificate contract should be updated"
    );
}

#[test]
fn test_set_cert_mgmt_contract_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    let cert_contract = Address::generate(&env);

    let supply_chain_id = env.register(SupplyChainTrackingContract, ());
    let supply_chain_client = SupplyChainTrackingContractClient::new(&env, &supply_chain_id);

    // Initialize
    supply_chain_client.initialize(&admin, &cert_contract.clone());

    // Try to update with unauthorized user
    let result = supply_chain_client.try_set_cert_mgmt_contract(&unauthorized_user, &cert_contract);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::UnauthorizedAccess)),
        "Should fail with unauthorized access"
    );
}

// =====================================================================================
// PRODUCT REGISTRATION TESTS
// =====================================================================================

#[test]
fn test_register_product_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Test");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Verify product details
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(product.farmer_id, farmer, "Farmer ID should match");
    assert_eq!(product.stages.len(), 0, "Should have no stages initially");
    assert_eq!(
        product.certificate_id,
        CertificateId::None,
        "Should have no certificate initially"
    );

    // Verify product registration details
    let registration = supply_chain_client.get_product_registration(&product_id);
    assert_eq!(
        registration.product_type, product_type,
        "Product type should match"
    );
    assert_eq!(
        registration.batch_number, batch_number,
        "Batch number should match"
    );
    assert_eq!(
        registration.origin_location, origin_location,
        "Origin location should match"
    );
    assert_eq!(
        registration.metadata_hash, metadata_hash,
        "Metadata hash should match"
    );

    // Verify farmer's product list
    let farmer_products = supply_chain_client.list_products_by_farmer(&farmer);
    assert_eq!(farmer_products.len(), 1, "Farmer should have one product");
    assert_eq!(
        farmer_products.get(0),
        Some(product_id.clone()),
        "Product ID should match"
    );

    // Verify product type index
    let products_by_type = supply_chain_client.list_products_by_type(&product_type);
    assert_eq!(
        products_by_type.len(),
        1,
        "Should have one product of this type"
    );
    assert_eq!(
        products_by_type.get(0),
        Some(product_id.clone()),
        "Product ID should match"
    );

    // // Verify events
    // let events = env.events().all();
    // let product_registered_topic = (Symbol::new(&env, "product_registered"), farmer.clone());
    // let farmer_products_updated_topic =
    //     (Symbol::new(&env, "farmer_products_updated"), farmer.clone());
    // let farmer_product_list_updated_topic = Symbol::new(&env, "farmer_product_list_updated");

    // assert!(
    //     events
    //         .iter()
    //         .any(|(_, topic, _)| topic == &product_registered_topic.into_val(&env)),
    //     "Should emit product_registered event"
    // );
    // assert!(
    //     events
    //         .iter()
    //         .any(|(_, topic, _)| topic == &farmer_products_updated_topic.into_val(&env)),
    //     "Should emit farmer_products_updated event"
    // );
    // assert!(
    //     events.iter().any(|(_, topic, _)| topic.get(&0)
    //         == &farmer_product_list_updated_topic.into_val(&env)),
    //     "Should emit farmer_product_list_updated event"
    // );
}

#[test]
fn test_register_product_invalid_input() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Test empty product type
    let result = supply_chain_client.try_register_product(
        &farmer,
        &String::from_str(&env, ""),
        &String::from_str(&env, "BATCH001"),
        &String::from_str(&env, "Farm Location"),
        &metadata_hash,
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidInput)),
        "Should fail with empty product type"
    );

    // Test empty batch number
    let result = supply_chain_client.try_register_product(
        &farmer,
        &String::from_str(&env, "Tomatoes"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Farm Location"),
        &metadata_hash,
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidInput)),
        "Should fail with empty batch number"
    );

    // Test empty origin location
    let result = supply_chain_client.try_register_product(
        &farmer,
        &String::from_str(&env, "Tomatoes"),
        &String::from_str(&env, "BATCH001"),
        &String::from_str(&env, ""),
        &metadata_hash,
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidInput)),
        "Should fail with empty origin location"
    );
}

#[test]
fn test_register_duplicate_product() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Test");

    // Register product first time
    supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Try to register same product again
    let result = supply_chain_client.try_register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::DuplicateProduct)),
        "Should fail with duplicate product"
    );
}

#[test]
fn test_multiple_farmers_multiple_products() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);

    // Register multiple products for farmer1
    let (product_type1, batch_number1, origin_location1, metadata_hash1) =
        create_test_product_data(&env, "Farmer1_Product1");
    let (product_type2, batch_number2, origin_location2, metadata_hash2) =
        create_test_product_data(&env, "Farmer1_Product2");

    let product1_id = supply_chain_client.register_product(
        &farmer1,
        &product_type1,
        &batch_number1,
        &origin_location1,
        &metadata_hash1,
    );
    let product2_id = supply_chain_client.register_product(
        &farmer1,
        &product_type2,
        &batch_number2,
        &origin_location2,
        &metadata_hash2,
    );

    // Register product for farmer2
    let (product_type3, batch_number3, origin_location3, metadata_hash3) =
        create_test_product_data(&env, "Farmer2_Product1");
    let product3_id = supply_chain_client.register_product(
        &farmer2,
        &product_type3,
        &batch_number3,
        &origin_location3,
        &metadata_hash3,
    );

    // Verify farmer1 has 2 products
    let farmer1_products = supply_chain_client.list_products_by_farmer(&farmer1);
    assert_eq!(farmer1_products.len(), 2, "Farmer1 should have 2 products");
    assert!(
        farmer1_products.contains(&product1_id),
        "Should contain product1"
    );
    assert!(
        farmer1_products.contains(&product2_id),
        "Should contain product2"
    );

    // Verify farmer2 has 1 product
    let farmer2_products = supply_chain_client.list_products_by_farmer(&farmer2);
    assert_eq!(farmer2_products.len(), 1, "Farmer2 should have 1 product");
    assert!(
        farmer2_products.contains(&product3_id),
        "Should contain product3"
    );
}

// =====================================================================================
// STAGE TRACKING TESTS
// =====================================================================================

#[test]
fn test_add_stage_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Stage");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Add first stage
    let stage_name = String::from_str(&env, "Harvesting");
    let location = String::from_str(&env, "Field 1");
    let data_hash = BytesN::from_array(&env, &[3u8; 32]);

    env.ledger().with_mut(|li| {
        li.timestamp += 3600; // Advance past initilization time
    });

    let stage_id = supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &stage_name,
        &location,
        &handler,
        &data_hash,
    );
    assert_eq!(stage_id, 1, "First stage should have ID 1");

    // Verify stage details
    let stage = supply_chain_client.get_stage_by_id(&product_id, &stage_id);
    assert_eq!(stage.stage_id, 1, "Stage ID should match");
    assert_eq!(stage.tier, StageTier::Planting, "Stage tier should match");
    assert_eq!(stage.name, stage_name, "Stage name should match");
    assert_eq!(stage.location, location, "Stage location should match");
    assert_eq!(stage.data_hash, data_hash, "Stage data hash should match");
    assert!(stage.timestamp > 0, "Stage timestamp should be set");

    // Verify product has the stage
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(product.stages.len(), 1, "Product should have 1 stage");
    assert_eq!(
        product.stages.get(0),
        Some(stage.clone()),
        "Stage should match"
    );

    // Verify current stage
    let current_stage = supply_chain_client.get_current_stage(&product_id);
    assert_eq!(current_stage, stage, "Current stage should match");

    // Add second stage
    let stage_name2 = String::from_str(&env, "Processing");
    let location2 = String::from_str(&env, "Processing Plant");
    let data_hash2 = BytesN::from_array(&env, &[4u8; 32]);

    let stage_id2 = supply_chain_client.add_stage(
        &product_id,
        &StageTier::Cultivation,
        &stage_name2,
        &location2,
        &handler,
        &data_hash2,
    );
    assert_eq!(stage_id2, 2, "Second stage should have ID 2");

    // Verify product now has 2 stages
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(product.stages.len(), 2, "Product should have 2 stages");
}

#[test]
fn test_add_stage_product_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let non_existent_product_id = BytesN::from_array(&env, &[99u8; 32]);

    let result = supply_chain_client.try_add_stage(
        &non_existent_product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Stage"),
        &String::from_str(&env, "Location"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::ProductNotFound)),
        "Should fail with product not found"
    );
}

#[test]
fn test_stage_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Validation");

    // Register product and add stages
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Stage 1"),
        &String::from_str(&env, "Location 1"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    // Test valid transition (1 -> 2)
    let is_valid = supply_chain_client.validate_stage_transition(&product_id, &1, &2);
    assert_eq!(is_valid, true, "Transition 1->2 should be valid");

    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Cultivation,
        &String::from_str(&env, "Stage 2"),
        &String::from_str(&env, "Location 2"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );

    // Test valid transition (2 -> 3)
    let is_valid = supply_chain_client.validate_stage_transition(&product_id, &2, &3);
    assert_eq!(is_valid, true, "Transition 2->3 should be valid");

    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Harvesting,
        &String::from_str(&env, "Stage 3"),
        &String::from_str(&env, "Location 3"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );

    // Test invalid transition (1 -> 3, skipping 2)
    let is_valid = supply_chain_client.validate_stage_transition(&product_id, &1, &3);
    assert_eq!(is_valid, false, "Transition 1->3 should be invalid");

    // Test transition from non-existent stage
    let result = supply_chain_client.try_validate_stage_transition(&product_id, &5, &6);
    assert!(result.is_err(), "Should fail for non-existent stage");
}

#[test]
fn test_get_stage_history() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "History");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Add stages in proper tier progression
    let stage_names = [
        String::from_str(&env, "Planting"),
        String::from_str(&env, "Growing"),
        String::from_str(&env, "Harvesting"),
        String::from_str(&env, "Processing"),
    ];
    let stage_locations = [
        String::from_str(&env, "Field A"),
        String::from_str(&env, "Field A"),
        String::from_str(&env, "Field A"),
        String::from_str(&env, "Processing Plant"),
    ];
    let stage_tiers = [
        StageTier::Planting,
        StageTier::Cultivation,
        StageTier::Harvesting,
        StageTier::Processing,
    ];

    for i in 0..4 {
        supply_chain_client.add_stage(
            &product_id,
            &stage_tiers[i],
            &stage_names[i],
            &stage_locations[i],
            &handler,
            &BytesN::from_array(&env, &[1u8; 32]),
        );
    }

    // Get stage history
    let history = supply_chain_client.get_stage_history(&product_id);
    assert_eq!(history.len(), 4, "Should have 4 stages in history");

    // Verify stage sequence and tiers
    for (i, stage) in history.iter().enumerate() {
        assert_eq!(
            stage.stage_id,
            (i + 1) as u32,
            "Stage ID should match sequence"
        );
        assert_eq!(stage.name, stage_names[i], "Stage name should match");
        assert_eq!(
            stage.location, stage_locations[i],
            "Stage location should match"
        );
        assert_eq!(stage.tier, stage_tiers[i], "Stage tier should match");
    }
}

#[test]
fn test_get_product_trace() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Trace");

    // Register product and add stages
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Harvesting"),
        &String::from_str(&env, "Farm"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Cultivation,
        &String::from_str(&env, "Processing"),
        &String::from_str(&env, "Plant"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );

    // Get product trace
    let (product, stages) = supply_chain_client.get_product_trace(&product_id);

    assert_eq!(product.farmer_id, farmer, "Product farmer should match");
    assert_eq!(stages.len(), 2, "Should have 2 stages");
    assert_eq!(
        stages.get(0).as_ref().unwrap().name,
        String::from_str(&env, "Harvesting"),
        "First stage should be harvesting"
    );
    assert_eq!(
        stages.get(1).as_ref().unwrap().name,
        String::from_str(&env, "Processing"),
        "Second stage should be processing"
    );
}

#[test]
fn test_get_stage_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "NotFound");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Add one stage
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Stage 1"),
        &String::from_str(&env, "Location"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    // Try to get non-existent stage
    let result = supply_chain_client.try_get_stage_by_id(&product_id, &999);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::StageNotFound)),
        "Should fail with stage not found"
    );
}

// =====================================================================================
// STAGE TIER VALIDATION TESTS
// =====================================================================================

#[test]
fn test_add_stage_wrong_tier_progression() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "WrongTier");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Test 1: Try to start with wrong tier (should start with Planting)
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Processing,
        &String::from_str(&env, "Processing"),
        &String::from_str(&env, "Processing Plant"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidTierProgression)),
        "Should fail when not starting with Planting tier"
    );

    // Add correct first stage (Planting)
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Planting Seeds"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    // Test 2: Try to skip a tier (Planting -> Harvesting, skipping Cultivation)
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Harvesting,
        &String::from_str(&env, "Harvesting Crops"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidTierProgression)),
        "Should fail when skipping Cultivation tier"
    );

    // Test 3: Try to go backwards (Planting -> Planting again)
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "More Planting"),
        &String::from_str(&env, "Another Field"),
        &handler,
        &BytesN::from_array(&env, &[3u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::DuplicateStageTier)),
        "Should fail when trying to add duplicate tier"
    );

    // Verify only one stage exists
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(
        product.stages.len(),
        1,
        "Should still have only 1 stage after failed attempts"
    );
    assert_eq!(
        product.stages.get(0).unwrap().tier,
        StageTier::Planting,
        "Should only have Planting stage"
    );
}

#[test]
fn test_add_stage_complete_wrong_sequence() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "WrongSequence");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Add stages in correct order up to Cultivation
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Planting"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Cultivation,
        &String::from_str(&env, "Growing"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );

    // Now try various wrong progressions from Cultivation

    // Test 1: Try to jump to Packaging (skipping Harvesting and Processing)
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Packaging,
        &String::from_str(&env, "Packaging"),
        &String::from_str(&env, "Packaging Facility"),
        &handler,
        &BytesN::from_array(&env, &[3u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidTierProgression)),
        "Should fail when jumping from Cultivation to Packaging"
    );

    // Test 2: Try to go back to Planting
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Re-planting"),
        &String::from_str(&env, "New Field"),
        &handler,
        &BytesN::from_array(&env, &[4u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::DuplicateStageTier)),
        "Should fail when trying to add duplicate Planting tier"
    );

    // Test 3: Try to add Cultivation again
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Cultivation,
        &String::from_str(&env, "More Cultivation"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[5u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::DuplicateStageTier)),
        "Should fail when trying to add duplicate Cultivation tier"
    );

    // Verify correct next tier is expected
    let next_tier = supply_chain_client.get_next_expected_tier(&product_id);
    assert_eq!(
        next_tier,
        Some(StageTier::Harvesting),
        "Next expected tier should be Harvesting"
    );

    // Add correct next stage
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Harvesting,
        &String::from_str(&env, "Harvesting"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[6u8; 32]),
    );

    // Verify product now has 3 stages in correct order
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(
        product.stages.len(),
        3,
        "Should have 3 stages after correct progression"
    );
    assert_eq!(
        product.stages.get(0).unwrap().tier,
        StageTier::Planting,
        "First stage should be Planting"
    );
    assert_eq!(
        product.stages.get(1).unwrap().tier,
        StageTier::Cultivation,
        "Second stage should be Cultivation"
    );
    assert_eq!(
        product.stages.get(2).unwrap().tier,
        StageTier::Harvesting,
        "Third stage should be Harvesting"
    );
}

#[test]
fn test_add_stage_after_final_tier() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "FinalTier");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Add all stages up to Consumer (final stage)
    let all_tiers = [
        StageTier::Planting,
        StageTier::Cultivation,
        StageTier::Harvesting,
        StageTier::Processing,
        StageTier::Packaging,
        StageTier::Storage,
        StageTier::Transportation,
        StageTier::Distribution,
        StageTier::Retail,
        StageTier::Consumer,
    ];

    let stage_names = [
        "Stage 1", "Stage 2", "Stage 3", "Stage 4", "Stage 5", "Stage 6", "Stage 7", "Stage 8",
        "Stage 9", "Stage 10",
    ];

    let location_names = [
        "Location 1",
        "Location 2",
        "Location 3",
        "Location 4",
        "Location 5",
        "Location 6",
        "Location 7",
        "Location 8",
        "Location 9",
        "Location 10",
    ];

    for (i, tier) in all_tiers.iter().enumerate() {
        supply_chain_client.add_stage(
            &product_id,
            tier,
            &String::from_str(&env, stage_names[i]),
            &String::from_str(&env, location_names[i]),
            &handler,
            &BytesN::from_array(&env, &[(i + 1) as u8; 32]),
        );
    }

    // Verify we're at the final stage
    let current_tier = supply_chain_client.get_current_tier(&product_id);
    assert_eq!(
        current_tier,
        Some(StageTier::Consumer),
        "Should be at Consumer tier"
    );

    let next_tier = supply_chain_client.get_next_expected_tier(&product_id);
    assert_eq!(next_tier, None, "Should have no next tier after Consumer");

    // Try to add another stage after Consumer - should fail
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Planting, // Any tier should fail
        &String::from_str(&env, "Post-Consumer"),
        &String::from_str(&env, "Somewhere"),
        &handler,
        &BytesN::from_array(&env, &[99u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::DuplicateStageTier)),
        "Should fail when trying to add stage after Consumer (final tier)"
    );

    // Verify product still has exactly 10 stages
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(
        product.stages.len(),
        10,
        "Should still have exactly 10 stages"
    );
    assert_eq!(
        product.stages.get(9).unwrap().tier,
        StageTier::Consumer,
        "Final stage should be Consumer"
    );
}

#[test]
fn test_tier_validation_edge_cases() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "EdgeCases");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Test current tier when no stages exist
    let current_tier = supply_chain_client.get_current_tier(&product_id);
    assert_eq!(
        current_tier, None,
        "Should have no current tier when no stages exist"
    );

    // Test next expected tier when no stages exist
    let next_tier = supply_chain_client.get_next_expected_tier(&product_id);
    assert_eq!(
        next_tier,
        Some(StageTier::Planting),
        "Should expect Planting as first tier"
    );

    // Add first stage correctly
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Planting"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    // Test current and next tier after first stage
    let current_tier = supply_chain_client.get_current_tier(&product_id);
    assert_eq!(
        current_tier,
        Some(StageTier::Planting),
        "Should have Planting as current tier"
    );

    let next_tier = supply_chain_client.get_next_expected_tier(&product_id);
    assert_eq!(
        next_tier,
        Some(StageTier::Cultivation),
        "Should expect Cultivation as next tier"
    );

    // Try to add a stage with same tier but different name - should still fail
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Different Planting"),
        &String::from_str(&env, "Different Field"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::DuplicateStageTier)),
        "Should fail even with different name for same tier"
    );
}

#[test]
fn test_add_stage_large_tier_jumps() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "LargeTierJumps");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Test trying to start with final tier
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Consumer,
        &String::from_str(&env, "Consumer Stage"),
        &String::from_str(&env, "Consumer Location"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidTierProgression)),
        "Should fail when trying to start with Consumer tier"
    );

    // Add correct first stage
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Planting"),
        &String::from_str(&env, "Field"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    // Test jumping multiple tiers (Planting -> Storage, skipping 4 tiers)
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Storage,
        &String::from_str(&env, "Storage"),
        &String::from_str(&env, "Warehouse"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidTierProgression)),
        "Should fail when jumping from Planting to Storage"
    );

    // Test jumping to final tier (Planting -> Consumer)
    let result = supply_chain_client.try_add_stage(
        &product_id,
        &StageTier::Consumer,
        &String::from_str(&env, "Consumer"),
        &String::from_str(&env, "Consumer Location"),
        &handler,
        &BytesN::from_array(&env, &[3u8; 32]),
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::InvalidTierProgression)),
        "Should fail when jumping from Planting to Consumer"
    );

    // Verify only one stage exists
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(
        product.stages.len(),
        1,
        "Should still have only 1 stage after failed attempts"
    );
}

#[test]
fn test_add_stage_invalid_backwards_progression() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "BackwardsProgression");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Add stages up to Processing
    let stages = [
        (StageTier::Planting, "Planting"),
        (StageTier::Cultivation, "Growing"),
        (StageTier::Harvesting, "Harvesting"),
        (StageTier::Processing, "Processing"),
    ];

    for (i, (tier, name)) in stages.iter().enumerate() {
        supply_chain_client.add_stage(
            &product_id,
            tier,
            &String::from_str(&env, name),
            &String::from_str(&env, "Location"),
            &handler,
            &BytesN::from_array(&env, &[(i + 1) as u8; 32]),
        );
    }

    // Test going backwards to each previous tier
    let backwards_attempts = [
        (
            StageTier::Harvesting,
            "Should fail going back to Harvesting",
        ),
        (
            StageTier::Cultivation,
            "Should fail going back to Cultivation",
        ),
        (StageTier::Planting, "Should fail going back to Planting"),
    ];

    for (tier, error_msg) in backwards_attempts.iter() {
        let result = supply_chain_client.try_add_stage(
            &product_id,
            tier,
            &String::from_str(&env, "Backwards Stage"),
            &String::from_str(&env, "Location"),
            &handler,
            &BytesN::from_array(&env, &[99u8; 32]),
        );
        assert_eq!(
            result,
            Err(Ok(SupplyChainError::DuplicateStageTier)),
            "{}",
            error_msg
        );
    }

    // Verify product still has exactly 4 stages
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(
        product.stages.len(),
        4,
        "Should still have exactly 4 stages"
    );
    assert_eq!(
        product.stages.get(3).unwrap().tier,
        StageTier::Processing,
        "Last stage should be Processing"
    );
}

#[test]
fn test_add_stage_tier_validation_with_get_functions() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "TierValidationWithGet");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Test progression through multiple stages with validation
    let valid_progression = [
        StageTier::Planting,
        StageTier::Cultivation,
        StageTier::Harvesting,
        StageTier::Processing,
        StageTier::Packaging,
    ];

    for (i, tier) in valid_progression.iter().enumerate() {
        // Check expected next tier before adding
        let expected_next = supply_chain_client.get_next_expected_tier(&product_id);
        assert_eq!(
            expected_next,
            Some(tier.clone()),
            "Expected next tier should match current tier being added"
        );

        let stage_names = [
            "Stage 1", "Stage 2", "Stage 3", "Stage 4", "Stage 5", "Stage 6", "Stage 7", "Stage 8",
            "Stage 9", "Stage 10",
        ];

        let location_names = [
            "Location 1",
            "Location 2",
            "Location 3",
            "Location 4",
            "Location 5",
            "Location 6",
            "Location 7",
            "Location 8",
            "Location 9",
            "Location 10",
        ];

        // Add the stage
        supply_chain_client.add_stage(
            &product_id,
            tier,
            &String::from_str(&env, &stage_names[i]),
            &String::from_str(&env, &location_names[i]),
            &handler,
            &BytesN::from_array(&env, &[(i + 1) as u8; 32]),
        );

        // Verify current tier after adding
        let current_tier = supply_chain_client.get_current_tier(&product_id);
        assert_eq!(
            current_tier,
            Some(tier.clone()),
            "Current tier should match the tier just added"
        );

        // Test that we can't add the same tier again
        let duplicate_result = supply_chain_client.try_add_stage(
            &product_id,
            tier,
            &String::from_str(&env, stage_names[i]),
            &String::from_str(&env, "Duplicate Location"),
            &handler,
            &BytesN::from_array(&env, &[99u8; 32]),
        );
        assert_eq!(
            duplicate_result,
            Err(Ok(SupplyChainError::DuplicateStageTier)),
            "Should fail when trying to add duplicate tier"
        );
    }

    // Verify final state
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(product.stages.len(), 5, "Should have exactly 5 stages");

    let current_tier = supply_chain_client.get_current_tier(&product_id);
    assert_eq!(
        current_tier,
        Some(StageTier::Packaging),
        "Current tier should be Packaging"
    );

    let next_tier = supply_chain_client.get_next_expected_tier(&product_id);
    assert_eq!(
        next_tier,
        Some(StageTier::Storage),
        "Next expected tier should be Storage"
    );
}

// =====================================================================================
// CERTIFICATE LINKING TESTS
// =====================================================================================

#[test]
fn test_link_certificate_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, authority, supply_chain_client, cert_client) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Cert");
    let (certificate_id, cert_status) = create_test_certificate_data(&env);

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    let cert_bytes = match &certificate_id {
        CertificateId::Some(bytes) => bytes.clone(),
        CertificateId::None => panic!("Expected CertificateId::Some variant"),
    };

    // Setup mock certificate
    setup_mock_certificate(
        &cert_client,
        &farmer,
        &authority,
        &cert_bytes,
        cert_status,
        cert_bytes.clone(),
    );

    // Initially no certificate linked
    let linked_cert = supply_chain_client.get_linked_certificate(&product_id);
    assert_eq!(
        linked_cert,
        CertificateId::None,
        "Should have no certificate initially"
    );

    // Link certificate
    supply_chain_client.link_certificate(&product_id, &certificate_id, &authority);

    // Verify certificate is linked
    let linked_cert = supply_chain_client.get_linked_certificate(&product_id);
    assert_eq!(
        linked_cert,
        CertificateId::Some(cert_bytes.clone()),
        "Certificate should be linked"
    );

    // Verify in product details
    let product = supply_chain_client.get_product_details(&product_id);
    assert_eq!(
        product.certificate_id,
        certificate_id.clone(),
        "Product should have certificate ID"
    );
}

#[test]
fn test_link_certificate_product_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, authority, supply_chain_client, _) = setup_test_environment(&env);
    let non_existent_product_id = BytesN::from_array(&env, &[99u8; 32]);
    let certificate_id = CertificateId::Some(BytesN::from_array(&env, &[2u8; 32]));

    let result = supply_chain_client.try_link_certificate(
        &non_existent_product_id,
        &certificate_id,
        &authority,
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::ProductNotFound)),
        "Should fail with product not found"
    );
}

#[test]
fn test_link_invalid_certificate() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, authority, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "InvalidCert");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Try to link invalid certificate (all zeros)
    let invalid_certificate_id = CertificateId::Some(BytesN::from_array(&env, &[0u8; 32]));
    let result =
        supply_chain_client.try_link_certificate(&product_id, &invalid_certificate_id, &authority);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::CertificateNotFound)),
        "Should fail with invalid certificate"
    );
}

#[test]
fn test_link_certificate_invalid_status() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, authority, supply_chain_client, cert_client) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "InvalidStatus");

    // Register product
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    let certificate_id = CertificateId::Some(BytesN::from_array(&env, &[3u8; 32]));
    let cert_bytes = match &certificate_id {
        CertificateId::Some(bytes) => bytes.clone(),
        CertificateId::None => panic!("Expected CertificateId::Some variant"),
    };

    // Setup mock certificate with EXPIRED status
    setup_mock_certificate(
        &cert_client,
        &farmer,
        &authority,
        &cert_bytes,
        CertStatus::Expired, // Invalid status
        cert_bytes.clone(),
    );

    // Try to link expired certificate - should fail
    let result = supply_chain_client.try_link_certificate(&product_id, &certificate_id, &authority);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::CertificateInvalid)),
        "Should fail with expired certificate"
    );

    // Setup mock certificate with REVOKED status
    setup_mock_certificate(
        &cert_client,
        &farmer,
        &authority,
        &cert_bytes,
        CertStatus::Revoked, // Invalid status
        cert_bytes.clone(),
    );

    // Try to link revoked certificate - should fail
    let result = supply_chain_client.try_link_certificate(&product_id, &certificate_id, &authority);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::CertificateInvalid)),
        "Should fail with revoked certificate"
    );

    // Verify no certificate was linked
    let linked_cert = supply_chain_client.get_linked_certificate(&product_id);
    assert_eq!(
        linked_cert,
        CertificateId::None,
        "No certificate should be linked after failed attempts"
    );
}

// =====================================================================================
// VERIFICATION AND VALIDATION TESTS
// =====================================================================================

#[test]
fn test_verify_authenticity_without_certificate() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Auth");

    // Register product and add stages
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Harvesting"),
        &String::from_str(&env, "Farm"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    // Calculate the actual supply chain hash using the contract's own hash calculation
    let test_hash = create_test_validation_hash(&env, &farmer, &product_id, 1u8);
    let is_authentic = supply_chain_client.verify_authenticity(&farmer, &product_id, &test_hash);
    assert!(
        is_authentic == true,
        "Should return a boolean result for authenticity verification"
    );

    // Test with definitively wrong hash - should return false
    let wrong_verification_data = BytesN::from_array(&env, &[99u8; 32]);
    let is_not_authentic =
        supply_chain_client.verify_authenticity(&farmer, &product_id, &wrong_verification_data);

    assert_eq!(
        is_not_authentic, false,
        "Should not be authentic with wrong hash"
    );
}

#[test]
fn test_verify_authenticity_with_certificate() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, authority, supply_chain_client, cert_client) =
        setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "AuthCert");
    let (certificate_id, cert_status) = create_test_certificate_data(&env);

    // Register product and add stages
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Harvesting"),
        &String::from_str(&env, "Farm"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    let cert_bytes = match &certificate_id {
        CertificateId::Some(bytes) => bytes.clone(),
        CertificateId::None => panic!("Expected CertificateId::Some variant"),
    };

    // Setup and link certificate using test hash
    let test_hash = create_test_validation_hash(&env, &farmer, &product_id, 1u8);
    setup_mock_certificate(
        &cert_client,
        &farmer,
        &authority,
        &cert_bytes,
        cert_status,
        test_hash.clone(),
    );
    supply_chain_client.link_certificate(&product_id, &certificate_id, &authority);

    let is_authentic = supply_chain_client.verify_authenticity(&farmer, &product_id, &test_hash);
    assert_eq!(
        is_authentic, true,
        "Should be authentic with correct certificate hash"
    );

    // Test with wrong certificate hash that doesn't match the stored certificate hash
    let wrong_cert_hash = BytesN::from_array(&env, &[99u8; 32]);
    let result =
        supply_chain_client.try_verify_authenticity(&farmer, &product_id, &wrong_cert_hash);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::VerificationHashInvalid)),
        "Should fail with wrong certificate hash"
    );
}

#[test]
fn test_verify_authenticity_product_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let non_existent_product_id = BytesN::from_array(&env, &[99u8; 32]);
    let verification_data = BytesN::from_array(&env, &[1u8; 32]);

    let result = supply_chain_client.try_verify_authenticity(
        &farmer,
        &non_existent_product_id,
        &verification_data,
    );
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::ProductNotFound)),
        "Should fail with product not found"
    );
}

#[test]
fn test_validate_certificate_integrity() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, authority, supply_chain_client, cert_client) = setup_test_environment(&env);
    let (certificate_id, cert_status) = create_test_certificate_data(&env);

    let cert_bytes = match &certificate_id {
        CertificateId::Some(bytes) => bytes.clone(),
        CertificateId::None => panic!("Expected CertificateId::Some variant"),
    };

    // Setup mock certificate
    setup_mock_certificate(
        &cert_client,
        &farmer,
        &authority,
        &cert_bytes,
        cert_status,
        cert_bytes.clone(),
    );

    // Test certificate validation by trying to link it (which validates existence and status)
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "CertTest");

    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    // Valid certificate should link successfully
    let authority = Address::generate(&env);
    supply_chain_client.link_certificate(&product_id, &certificate_id, &authority);

    let linked_cert = supply_chain_client.get_linked_certificate(&product_id);
    assert_eq!(
        linked_cert, certificate_id,
        "Valid certificate should be linked"
    );

    // Test invalid certificate
    let invalid_cert_id = CertificateId::Some(BytesN::from_array(&env, &[0u8; 32]));
    let product_id2 = supply_chain_client.register_product(
        &farmer,
        &String::from_str(&env, "Test2"),
        &String::from_str(&env, "BATCH002"),
        &String::from_str(&env, "Farm2"),
        &metadata_hash,
    );

    let result =
        supply_chain_client.try_link_certificate(&product_id2, &invalid_cert_id, &authority);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::CertificateNotFound)),
        "Invalid certificate should fail validation"
    );
}

// =====================================================================================
// QR CODE FUNCTIONALITY TESTS
// =====================================================================================

#[test]
fn test_qr_code_generation_and_tracing() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "QR");

    // Register product and add stages
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Harvesting"),
        &String::from_str(&env, "Farm"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );

    // Generate QR code
    let qr_code = supply_chain_client.generate_qr_code(&product_id);
    assert!(qr_code.len() > 0, "QR code should be generated");

    // Use QR code to trace product
    let (traced_product, traced_stages) = supply_chain_client.trace_by_qr_code(&qr_code);
    assert_eq!(
        traced_product.product_id, product_id,
        "Traced product ID should match"
    );
    assert_eq!(
        traced_product.farmer_id, farmer,
        "Traced farmer should match"
    );
    assert_eq!(traced_stages.len(), 1, "Should have 1 stage");
}

#[test]
fn test_trace_by_invalid_qr_code() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let invalid_qr_code = String::from_str(&env, "invalid_qr_code");

    let result = supply_chain_client.try_trace_by_qr_code(&invalid_qr_code);
    assert!(result.is_err(), "Should fail with invalid QR code");
}

// =====================================================================================
// HASH CHAIN VERIFICATION TESTS
// =====================================================================================

#[test]
fn test_verify_hash_chain() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, handler, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "Hash");

    // Register product and add stages
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Planting,
        &String::from_str(&env, "Stage1"),
        &String::from_str(&env, "Location1"),
        &handler,
        &BytesN::from_array(&env, &[1u8; 32]),
    );
    supply_chain_client.add_stage(
        &product_id,
        &StageTier::Cultivation,
        &String::from_str(&env, "Stage2"),
        &String::from_str(&env, "Location2"),
        &handler,
        &BytesN::from_array(&env, &[2u8; 32]),
    );

    // Verify hash chain
    let is_valid = supply_chain_client.verify_hash_chain(&product_id);
    assert_eq!(is_valid, true, "Hash chain should be valid");
}

#[test]
fn test_verify_hash_chain_product_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let non_existent_product_id = BytesN::from_array(&env, &[99u8; 32]);
    let result = supply_chain_client.try_verify_hash_chain(&non_existent_product_id);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::ProductNotFound)),
        "Should fail with product not found"
    );
}

// =====================================================================================
// EDGE CASES AND ERROR HANDLING TESTS
// =====================================================================================

#[test]
fn test_get_product_details_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let non_existent_product_id = BytesN::from_array(&env, &[99u8; 32]);

    let result = supply_chain_client.try_get_product_details(&non_existent_product_id);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::ProductNotFound)),
        "Should fail with product not found"
    );
}

#[test]
fn test_get_product_registration_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let non_existent_product_id = BytesN::from_array(&env, &[99u8; 32]);

    let result = supply_chain_client.try_get_product_registration(&non_existent_product_id);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::ProductNotFound)),
        "Should fail with product not found"
    );
}

#[test]
fn test_list_products_empty_farmer() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let farmer_with_no_products = Address::generate(&env);

    let products = supply_chain_client.list_products_by_farmer(&farmer_with_no_products);
    assert_eq!(
        products.len(),
        0,
        "Should return empty list for farmer with no products"
    );
}

#[test]
fn test_list_products_by_type_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let non_existent_product_type = String::from_str(&env, "NonExistentType");

    let products = supply_chain_client.list_products_by_type(&non_existent_product_type);
    assert_eq!(
        products.len(),
        0,
        "Should return empty list for non-existent product type"
    );
}

#[test]
fn test_get_current_stage_no_stages() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, farmer, _, _, supply_chain_client, _) = setup_test_environment(&env);
    let (product_type, batch_number, origin_location, metadata_hash) =
        create_test_product_data(&env, "NoStages");

    // Register product without adding stages
    let product_id = supply_chain_client.register_product(
        &farmer,
        &product_type,
        &batch_number,
        &origin_location,
        &metadata_hash,
    );

    let result = supply_chain_client.try_get_current_stage(&product_id);
    assert_eq!(
        result,
        Err(Ok(SupplyChainError::StageNotFound)),
        "Should fail when no stages exist"
    );
}

// =====================================================================================
// MOCK CERTIFICATE MANAGEMENT CONTRACT
// =====================================================================================

#[contract]
struct MockCertificateManagement;

#[contractimpl]
impl MockCertificateManagement {
    pub fn set_certification(env: Env, owner: Address, cert_id: u32, certification: Certification) {
        let key = Symbol::new(&env, "certification");
        let mut data: Map<(Address, u32), Certification> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));
        data.set((owner.clone(), cert_id), certification.clone());
        env.storage().instance().set(&key, &data);
        log!(
            &env,
            "Set certification: owner={:?}, cert_id={}, cert={:?}",
            owner,
            cert_id,
            certification
        );
    }

    pub fn get_cert(
        env: Env,
        owner: Address,
        cert_id: u32,
    ) -> Result<Certification, CertificationError> {
        let key = Symbol::new(&env, "certification");
        let data: Map<(Address, u32), Certification> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));
        let result = data
            .get((owner.clone(), cert_id))
            .ok_or(CertificationError::NotFound)?;
        log!(
            &env,
            "Get cert: owner={:?}, cert_id={}, cert={:?}",
            owner,
            cert_id,
            result
        );
        Ok(result)
    }

    pub fn set_cert_status(env: Env, owner: Address, cert_id: u32, status: CertStatus) {
        let key = Symbol::new(&env, "cert_status");
        let mut data: Map<(Address, u32), CertStatus> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));
        data.set((owner.clone(), cert_id), status.clone());
        env.storage().instance().set(&key, &data);
        log!(
            &env,
            "Set cert status: owner={:?}, cert_id={}, status={:?}",
            owner,
            cert_id,
            status
        );
    }

    pub fn check_cert_status(
        env: Env,
        owner: Address,
        cert_id: u32,
    ) -> Result<CertStatus, CertificationError> {
        let key = Symbol::new(&env, "cert_status");
        let data: Map<(Address, u32), CertStatus> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));
        let result = data
            .get((owner.clone(), cert_id))
            .ok_or(CertificationError::NotFound)?;
        log!(
            &env,
            "Check cert status: owner={:?}, cert_id={}, status={:?}",
            owner,
            cert_id,
            result
        );
        Ok(result)
    }

    pub fn set_cert_verification_hash(env: Env, owner: Address, cert_id: u32, hash: BytesN<32>) {
        let key = Symbol::new(&env, "cert_hash");
        let mut data: Map<(Address, u32), BytesN<32>> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));
        data.set((owner.clone(), cert_id), hash.clone());
        env.storage().instance().set(&key, &data);
        log!(
            &env,
            "Set cert hash: owner={:?}, cert_id={}, hash={:?}",
            owner,
            cert_id,
            hash
        );
    }

    pub fn verify_document_hash(
        env: Env,
        owner: Address,
        cert_id: u32,
        submitted_hash: BytesN<32>,
    ) -> Result<(), VerifyError> {
        let key = Symbol::new(&env, "cert_hash");
        let data: Map<(Address, u32), BytesN<32>> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(&env));
        let stored_hash = data
            .get((owner.clone(), cert_id))
            .ok_or(VerifyError::NotFound)?;

        if stored_hash != submitted_hash {
            return Err(VerifyError::HashMismatch);
        }

        Ok(())
    }
}
