//! Comprehensive tests for certificate issuance and validation functionality

#![cfg(test)]

use crate::{
    error::ContractError,
    certification::issue_certificate,
    tests::utils::*,
};
use soroban_sdk::{BytesN, String, Address, Symbol};

/// Test module for successful certificate issuance
mod certificate_issuance {
    use super::*;

    #[test]
    fn test_successful_certificate_issuance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        // Setup complete scenario with external contracts
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let program_id = &scenario.programs[0];
        let farmer = &scenario.participants[0];
        
        // Enroll farmer and complete program
        crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        crate::participation::update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
        
        // Issue certificate
        let result = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone());
        
        assert!(result.is_ok(), "Certificate issuance should succeed");
        let certificate_id = result.unwrap();
        
        // Verify certificate was issued and recorded
        let updated_program = crate::storage::get_program(&env.env, program_id).unwrap();
        let participant_status = updated_program.participants.get(farmer.clone()).unwrap();
        
        assert_eq!(participant_status.certificate_id, certificate_id, "Certificate ID should be recorded in participant status");
        assert_ne!(participant_status.certificate_id, BytesN::from_array(&env.env, &[0; 32]), "Certificate ID should not be zero");
    }

    #[test]
    fn test_certificate_issuance_unauthorized_instructor() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let authorized_instructor = &scenario.instructors[0];
        let unauthorized_instructor = factory.mock_instructor("unauthorized_cert_instructor");
        let program_id = &scenario.programs[0];
        let farmer = &scenario.participants[0];
        
        // Setup completed farmer
        crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        crate::participation::update_progress(&env.env, authorized_instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
        
        // Try to issue certificate with unauthorized instructor
        let result = issue_certificate(&env.env, unauthorized_instructor.clone(), program_id.clone(), farmer.clone());
        
        TestAssertions::assert_contract_error(result, ContractError::NotInstructor);
    }

    #[test]
    fn test_certificate_issuance_incomplete_program() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let program_id = &scenario.programs[0];
        let farmer = &scenario.participants[0];
        
        // Enroll farmer but don't complete program (< 100% progress)
        crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        crate::participation::update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 85).unwrap();
        
        // Try to issue certificate for incomplete program
        let result = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone());
        
        TestAssertions::assert_contract_error(result, ContractError::NotCompleted);
    }

    #[test]
    fn test_certificate_issuance_unenrolled_farmer() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let program_id = &scenario.programs[0];
        let unenrolled_farmer = factory.mock_farmer("unenrolled_cert_farmer");
        
        // Try to issue certificate for farmer not enrolled in program
        let result = issue_certificate(&env.env, instructor.clone(), program_id.clone(), unenrolled_farmer);
        
        TestAssertions::assert_contract_error(result, ContractError::ParticipantNotFound);
    }

    #[test]
    fn test_duplicate_certificate_issuance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let program_id = &scenario.programs[0];
        let farmer = &scenario.participants[0];
        
        // Setup completed farmer
        crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        crate::participation::update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
        
        // Issue certificate first time
        let first_result = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone());
        assert!(first_result.is_ok(), "First certificate issuance should succeed");
        
        // Try to issue certificate again (should fail)
        let second_result = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone());
        
        TestAssertions::assert_contract_error(second_result, ContractError::AlreadyCertified);
    }

    #[test]
    fn test_certificate_issuance_nonexistent_program() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        helper.setup_complete_scenario(); // Setup external contracts
        
        let instructor = factory.mock_instructor("nonexistent_program_instructor");
        let farmer = factory.mock_farmer("nonexistent_program_farmer");
        let nonexistent_program_id = factory.mock_program_id("nonexistent_program");
        
        let result = issue_certificate(&env.env, instructor, nonexistent_program_id, farmer);
        
        TestAssertions::assert_contract_error(result, ContractError::ProgramNotFound);
    }

    #[test]
    fn test_certificate_id_uniqueness() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let program_id = &scenario.programs[0];
        
        // Setup multiple farmers for the same program
        let farmers = vec![
            factory.mock_farmer("unique_cert_farmer_1"),
            factory.mock_farmer("unique_cert_farmer_2"),
            factory.mock_farmer("unique_cert_farmer_3"),
        ];
        
        let mut certificate_ids = Vec::new();
        
        for farmer in farmers.iter() {
            // Enroll and complete program
            crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
            crate::participation::update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
            
            // Issue certificate
            let certificate_id = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone()).unwrap();
            certificate_ids.push(certificate_id);
        }
        
        // Verify all certificate IDs are unique
        for i in 0..certificate_ids.len() {
            for j in i + 1..certificate_ids.len() {
                assert_ne!(certificate_ids[i], certificate_ids[j], 
                          "Certificate IDs should be unique for different farmers");
            }
        }
    }

    #[test]
    fn test_certificate_issuance_different_programs() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let farmer = &scenario.participants[0];
        
        // Use multiple programs from scenario
        let mut certificate_ids = Vec::new();
        
        for program_id in scenario.programs.iter() {
            // Enroll and complete each program
            crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
            crate::participation::update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
            
            // Issue certificate
            let certificate_id = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone()).unwrap();
            certificate_ids.push(certificate_id);
        }
        
        // Verify farmer can receive certificates from multiple programs
        assert_eq!(certificate_ids.len(), scenario.programs.len());
        
        // Verify all certificates are unique
        for i in 0..certificate_ids.len() {
            for j in i + 1..certificate_ids.len() {
                assert_ne!(certificate_ids[i], certificate_ids[j], 
                          "Certificates from different programs should be unique");
            }
        }
    }
}

/// Test module for certificate data integrity and validation
mod certificate_data_integrity {
    use super::*;

    #[test]
    fn test_certificate_id_generation_deterministic() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let program_id = &scenario.programs[0];
        let farmer = &scenario.participants[0];
        
        // Setup completed farmer
        crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        crate::participation::update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
        
        // Generate expected certificate ID manually
        let expected_certificate_id = crate::utils::utils::generate_id(&env.env, (program_id.clone(), farmer.clone()));
        
        // Issue certificate
        let actual_certificate_id = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone()).unwrap();
        
        assert_eq!(actual_certificate_id, expected_certificate_id, "Certificate ID should be deterministic");
    }

    #[test]
    fn test_certificate_updates_participant_status() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let helper = IntegrationTestHelper::new(&env);
        
        let scenario = helper.setup_complete_scenario();
        
        let instructor = &scenario.instructors[0];
        let program_id = &scenario.programs[0];
        let farmer = &scenario.participants[0];
        
        // Setup completed farmer
        crate::participation::enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        crate::participation::update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
        
        // Get status before certificate issuance
        let program_before = crate::storage::get_program(&env.env, program_id).unwrap();
        let status_before = program_before.participants.get(farmer.clone()).unwrap();
        assert_eq!(status_before.certificate_id, BytesN::from_array(&env.env, &[0; 32]), "Should start with no certificate");
        
        // Issue certificate
        let certificate_id = issue_certificate(&env.env, instructor.clone(), program_id.clone(), farmer.clone()).unwrap();
        
        // Verify status was updated
        let program_after = crate::storage::get_program(&env.env, program_id).unwrap();
        let status_after = program_after.participants.get(farmer.clone()).unwrap();
        
        assert_eq!(status_after.certificate_id, certificate_id, "Certificate ID should be recorded");
        assert_eq!(status_after.farmer_id, farmer.clone(), "Farmer ID should be preserved");
        assert_eq!(status_after.progress, 100, "Progress should be preserved");
    }
}