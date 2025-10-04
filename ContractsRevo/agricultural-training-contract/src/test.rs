#![cfg(test)]

use super::*;
use crate::error::ContractError;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, IntoVal, String, Symbol,
};

// --- Mock Contracts for Testing Dependencies ---

// A mock for the Certificate Management Contract
#[contract]
pub struct MockCertificateContract;

#[contractimpl]
impl CertificateManagementContract for MockCertificateContract {
    fn issue_certification(
        env: Env,
        issuer: Address,
        recipient: Address,
        cert_type: Symbol,
        _expiration_date: u64,
        verification_hash: BytesN<32>,
    ) {
        env.events().publish(
            (Symbol::new(&env, "cert_issued"), recipient),
            (issuer, cert_type, verification_hash),
        );
    }
}

// A mock for the Loyalty Token Contract
#[contract]
pub struct MockLoyaltyContract;

#[contractimpl]
impl LoyaltyTokenContract for MockLoyaltyContract {
    fn award_points(
        env: Env,
        program_id: BytesN<32>,
        user_address: Address,
        transaction_amount: u32,
    ) {
        env.events().publish(
            (Symbol::new(&env, "points_awarded"), user_address),
            (program_id, transaction_amount),
        );
    }
}

// --- Test ---

struct TrainingTest<'a> {
    env: Env,
    admin: Address,
    instructor: Address,
    farmer: Address,
    contract: AgriculturalTrainingContractClient<'a>,
    loyalty_program_id: BytesN<32>,
}

impl<'a> TrainingTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let farmer = Address::generate(&env);

        // Deploy and register mock contracts
        let certificate_contract_id = env.register_contract(None, MockCertificateContract);
        let loyalty_token_id = env.register_contract(None, MockLoyaltyContract);
        let loyalty_program_id = BytesN::random(&env);

        // Deploy the main contract
        let contract_id = env.register_contract(None, AgriculturalTrainingContract);
        let contract = AgriculturalTrainingContractClient::new(&env, &contract_id);

        // Initialize the main contract with the mock contract addresses
        contract.initialize(
            &admin,
            &certificate_contract_id,
            &loyalty_token_id,
            &loyalty_program_id,
        );

        TrainingTest {
            env,
            admin,
            instructor,
            farmer,
            contract,
            loyalty_program_id,
        }
    }
}

// --- Tests ---

#[test]
fn test_initialize() {
    let test = TrainingTest::setup();
    // Try to initialize again, should fail.
    let result = test.contract.try_initialize(
        &test.admin,
        &Address::generate(&test.env),
        &Address::generate(&test.env),
        &BytesN::random(&test.env),
    );
    assert_eq!(result, Err(Ok(ContractError::AlreadyInitialized)));
}

#[test]
fn test_create_training_program() {
    let test = TrainingTest::setup();
    let title = String::from_str(&test.env, "Organic Farming Basics");
    let program_id = test.contract.create_training_program(
        &test.instructor,
        &title,
        &"Description".into_val(&test.env),
        &40,
        &BytesN::random(&test.env),
    );

    let program = test.contract.get_program(&program_id);
    assert_eq!(program.title, title);
    assert_eq!(program.instructor_id, test.instructor);
    assert_eq!(program.participants.is_empty(), true);
}

#[test]
fn test_enroll_farmer() {
    let test = TrainingTest::setup();
    let program_id = test.contract.create_training_program(
        &test.instructor,
        &"T1".into_val(&test.env),
        &"D1".into_val(&test.env),
        &10,
        &BytesN::random(&test.env),
    );

    test.contract.enroll_farmer(&test.farmer, &program_id);
    let status = test
        .contract
        .get_participant_status(&program_id, &test.farmer);

    assert_eq!(status.progress, 0);
    assert_eq!(
        status.certificate_id,
        BytesN::from_array(&test.env, &[0; 32])
    );

    // Try to enroll again, should fail.
    let result = test.contract.try_enroll_farmer(&test.farmer, &program_id);
    assert_eq!(result, Err(Ok(ContractError::AlreadyEnrolled)));
}

#[test]
fn test_update_progress() {
    let test = TrainingTest::setup();
    let program_id = test.contract.create_training_program(
        &test.instructor,
        &"T1".into_val(&test.env),
        &"D1".into_val(&test.env),
        &10,
        &BytesN::random(&test.env),
    );
    test.contract.enroll_farmer(&test.farmer, &program_id);

    // Successful update by instructor
    test.contract
        .update_progress(&test.instructor, &program_id, &test.farmer, &50);
    let status = test
        .contract
        .get_participant_status(&program_id, &test.farmer);
    assert_eq!(status.progress, 50);

    // Unauthorized update by another user
    let another_user = Address::generate(&test.env);
    let result = test
        .contract
        .try_update_progress(&another_user, &program_id, &test.farmer, &75);
    assert_eq!(result, Err(Ok(ContractError::NotInstructor)));
}

#[test]
fn test_issue_certificate() {
    let test = TrainingTest::setup();
    let program_id = test.contract.create_training_program(
        &test.instructor,
        &"T1".into_val(&test.env),
        &"D1".into_val(&test.env),
        &10,
        &BytesN::random(&test.env),
    );
    test.contract.enroll_farmer(&test.farmer, &program_id);

    // Try to issue before completion, should fail.
    let result_not_completed =
        test.contract
            .try_issue_certificate(&test.instructor, &program_id, &test.farmer);
    assert_eq!(result_not_completed, Err(Ok(ContractError::NotCompleted)));

    // Complete the program
    test.contract
        .update_progress(&test.instructor, &program_id, &test.farmer, &100);

    // Successfully issue certificate
    let certificate_id =
        test.contract
            .issue_certificate(&test.instructor, &program_id, &test.farmer);

    // Verify status
    let status = test
        .contract
        .get_participant_status(&program_id, &test.farmer);
    assert_eq!(status.certificate_id, certificate_id);

    // Try to issue again, should fail.
    let result_already_certified =
        test.contract
            .try_issue_certificate(&test.instructor, &program_id, &test.farmer);
    assert_eq!(
        result_already_certified,
        Err(Ok(ContractError::AlreadyCertified))
    );
}
