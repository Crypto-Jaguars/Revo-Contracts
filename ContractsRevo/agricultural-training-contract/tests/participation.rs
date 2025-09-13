//! Comprehensive tests for farmer participation and progress tracking functionality

#![cfg(test)]

use crate::{
    error::ContractError,
    participation::{enroll_farmer, update_progress},
    tests::utils::*,
};
use soroban_sdk::{BytesN, String, Address};

/// Test module for farmer enrollment functionality
mod farmer_enrollment {
    use super::*;

    #[test]
    fn test_successful_farmer_enrollment() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Create a training program first
        let instructor = factory.mock_instructor("enrollment_instructor");
        let title = factory.create_string("Enrollment Test Program");
        let description = factory.create_string("Program for testing farmer enrollment");
        let duration_hours = 40u32;
        let materials_hash = factory.mock_materials_hash("enrollment_materials");
        
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            title,
            description,
            duration_hours,
            materials_hash,
        ).unwrap();
        
        // Enroll a farmer
        let farmer = factory.mock_farmer("test_farmer");
        let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
        
        assert!(result.is_ok(), "Farmer enrollment should succeed");
        
        // Verify farmer was enrolled correctly
        let updated_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(updated_program.participants.len(), 1, "Program should have one participant");
        
        let participant_status = updated_program.participants.get(farmer.clone()).unwrap();
        assert_eq!(participant_status.farmer_id, farmer);
        assert_eq!(participant_status.progress, 0, "Initial progress should be 0");
        assert_eq!(participant_status.certificate_id, BytesN::from_array(&env.env, &[0; 32]), "Certificate should be unissued initially");
    }

    #[test]
    fn test_farmer_enrollment_in_nonexistent_program() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let farmer = factory.mock_farmer("test_farmer");
        let nonexistent_program_id = factory.mock_program_id("nonexistent_program");
        
        let result = enroll_farmer(&env.env, farmer, nonexistent_program_id);
        
        // Should get ProgramNotFound error from storage::get_program
        TestAssertions::assert_contract_error(result, ContractError::ProgramNotFound);
    }

    #[test]
    fn test_duplicate_farmer_enrollment() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup program
        let instructor = factory.mock_instructor("duplicate_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor,
            factory.create_string("Duplicate Enrollment Test"),
            factory.create_string("Testing duplicate enrollment prevention"),
            30,
            factory.mock_materials_hash("duplicate_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("duplicate_farmer");
        
        // First enrollment should succeed
        let first_result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
        assert!(first_result.is_ok(), "First enrollment should succeed");
        
        // Second enrollment should fail
        let second_result = enroll_farmer(&env.env, farmer, program_id);
        TestAssertions::assert_contract_error(second_result, ContractError::AlreadyEnrolled);
    }

    #[test]
    fn test_multiple_farmers_enrollment() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup program
        let instructor = factory.mock_instructor("multi_enrollment_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor,
            factory.create_string("Multi-Farmer Test Program"),
            factory.create_string("Program for testing multiple farmer enrollments"),
            50,
            factory.mock_materials_hash("multi_farmer_materials"),
        ).unwrap();
        
        // Enroll multiple farmers
        let farmers = vec![
            factory.mock_farmer("farmer_alice"),
            factory.mock_farmer("farmer_bob"),
            factory.mock_farmer("farmer_carol"),
            factory.mock_farmer("farmer_david"),
            factory.mock_farmer("farmer_eve"),
        ];
        
        for farmer in farmers.iter() {
            let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
            assert!(result.is_ok(), "Each farmer enrollment should succeed");
        }
        
        // Verify all farmers are enrolled
        let updated_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(updated_program.participants.len(), farmers.len(), "All farmers should be enrolled");
        
        // Verify each farmer's initial status
        for farmer in farmers.iter() {
            let participant_status = updated_program.participants.get(farmer.clone()).unwrap();
            assert_eq!(participant_status.farmer_id, *farmer);
            assert_eq!(participant_status.progress, 0);
            assert_eq!(participant_status.certificate_id, BytesN::from_array(&env.env, &[0; 32]));
        }
    }

    #[test]
    fn test_farmer_enrollment_across_multiple_programs() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("cross_program_instructor");
        let farmer = factory.mock_farmer("cross_program_farmer");
        
        // Create multiple programs
        let programs = vec![
            ("Program A", "Description A", 30u32),
            ("Program B", "Description B", 40u32),
            ("Program C", "Description C", 50u32),
        ];
        
        let mut program_ids = Vec::new();
        
        for (title, desc, duration) in programs.iter() {
            env.advance_time(1);
            
            let program_id = crate::training::create_training_program(
                &env.env,
                instructor.clone(),
                factory.create_string(title),
                factory.create_string(desc),
                *duration,
                factory.mock_materials_hash(&format!("{}_materials", title)),
            ).unwrap();
            
            program_ids.push(program_id);
        }
        
        // Enroll same farmer in all programs
        for program_id in program_ids.iter() {
            let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
            assert!(result.is_ok(), "Farmer should be able to enroll in multiple programs");
        }
        
        // Verify farmer is enrolled in all programs
        for program_id in program_ids.iter() {
            let program = crate::storage::get_program(&env.env, program_id).unwrap();
            assert_eq!(program.participants.len(), 1, "Each program should have one participant");
            
            let participant_status = program.participants.get(farmer.clone()).unwrap();
            assert_eq!(participant_status.farmer_id, farmer);
        }
    }

    #[test]
    fn test_enrollment_preserves_program_integrity() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("integrity_instructor");
        let original_title = factory.create_string("Integrity Test Program");
        let original_description = factory.create_string("Testing program integrity during enrollment");
        let original_duration = 35u32;
        let original_materials = factory.mock_materials_hash("integrity_materials");
        
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            original_title.clone(),
            original_description.clone(),
            original_duration,
            original_materials.clone(),
        ).unwrap();
        
        // Get original program state
        let original_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        
        // Enroll farmer
        let farmer = factory.mock_farmer("integrity_farmer");
        let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
        assert!(result.is_ok());
        
        // Verify program metadata remained unchanged
        let updated_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(updated_program.program_id, original_program.program_id);
        assert_eq!(updated_program.title, original_title);
        assert_eq!(updated_program.description, original_description);
        assert_eq!(updated_program.duration_hours, original_duration);
        assert_eq!(updated_program.instructor_id, instructor);
        assert_eq!(updated_program.materials_hash, original_materials);
        
        // Only participants should have changed
        assert_eq!(original_program.participants.len(), 0);
        assert_eq!(updated_program.participants.len(), 1);
    }

    #[test]
    fn test_participant_status_initialization() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("status_init_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor,
            factory.create_string("Status Initialization Test"),
            factory.create_string("Testing participant status initialization"),
            45,
            factory.mock_materials_hash("status_init_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("status_init_farmer");
        let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
        assert!(result.is_ok());
        
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let status = program.participants.get(farmer.clone()).unwrap();
        
        // Verify proper initialization
        assert_eq!(status.farmer_id, farmer, "Farmer ID should match");
        assert_eq!(status.progress, 0, "Progress should start at 0");
        
        // Check that certificate_id is properly zeroed
        let zero_hash = BytesN::from_array(&env.env, &[0; 32]);
        assert_eq!(status.certificate_id, zero_hash, "Certificate ID should be zeroed initially");
        
        // Verify certificate is truly unissued (all bytes are zero)
        let cert_bytes = status.certificate_id.to_array();
        assert!(cert_bytes.iter().all(|&b| b == 0), "Certificate ID should be all zeros");
    }
}

/// Test module for progress tracking functionality
mod progress_tracking {
    use super::*;

    #[test]
    fn test_successful_progress_update() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup program and enroll farmer
        let instructor = factory.mock_instructor("progress_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Progress Test Program"),
            factory.create_string("Testing progress updates"),
            60,
            factory.mock_materials_hash("progress_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("progress_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Update progress
        let new_progress = 45u32;
        let result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), new_progress);
        
        assert!(result.is_ok(), "Progress update should succeed");
        
        // Verify progress was updated
        let updated_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let participant_status = updated_program.participants.get(farmer).unwrap();
        assert_eq!(participant_status.progress, new_progress, "Progress should be updated to new value");
    }

    #[test]
    fn test_progress_update_unauthorized_instructor() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup program with one instructor
        let authorized_instructor = factory.mock_instructor("authorized_instructor");
        let unauthorized_instructor = factory.mock_instructor("unauthorized_instructor");
        
        let program_id = crate::training::create_training_program(
            &env.env,
            authorized_instructor.clone(),
            factory.create_string("Authorization Test Program"),
            factory.create_string("Testing instructor authorization"),
            40,
            factory.mock_materials_hash("auth_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("auth_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Try to update progress with unauthorized instructor
        let result = update_progress(&env.env, unauthorized_instructor, program_id, farmer, 50);
        
        TestAssertions::assert_contract_error(result, ContractError::NotInstructor);
    }

    #[test]
    fn test_progress_update_unenrolled_farmer() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("unenrolled_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Unenrolled Test Program"),
            factory.create_string("Testing unenrolled farmer progress update"),
            35,
            factory.mock_materials_hash("unenrolled_materials"),
        ).unwrap();
        
        // Don't enroll the farmer
        let unenrolled_farmer = factory.mock_farmer("unenrolled_farmer");
        
        // Try to update progress for unenrolled farmer
        let result = update_progress(&env.env, instructor, program_id, unenrolled_farmer, 25);
        
        TestAssertions::assert_contract_error(result, ContractError::ParticipantNotFound);
    }

    #[test]
    fn test_progress_update_invalid_percentage_over_100() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup program and farmer
        let instructor = factory.mock_instructor("invalid_progress_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Invalid Progress Test"),
            factory.create_string("Testing invalid progress values"),
            50,
            factory.mock_materials_hash("invalid_progress_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("invalid_progress_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Try to update with invalid progress > 100
        let invalid_progress = 150u32;
        let result = update_progress(&env.env, instructor, program_id, farmer, invalid_progress);
        
        TestAssertions::assert_contract_error(result, ContractError::InvalidData);
    }

    #[test]
    fn test_progress_update_valid_boundary_values() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("boundary_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Boundary Values Test"),
            factory.create_string("Testing boundary progress values"),
            30,
            factory.mock_materials_hash("boundary_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("boundary_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Test valid boundary values
        let valid_progress_values = vec![0u32, 1u32, 50u32, 99u32, 100u32];
        
        for progress in valid_progress_values.iter() {
            let result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), *progress);
            assert!(result.is_ok(), "Progress {} should be valid", progress);
            
            // Verify progress was set correctly
            let program = crate::storage::get_program(&env.env, &program_id).unwrap();
            let status = program.participants.get(farmer.clone()).unwrap();
            assert_eq!(status.progress, *progress, "Progress should be set to {}", progress);
            
            TestAssertions::assert_valid_progress(*progress);
        }
    }

    #[test]
    fn test_progress_update_sequence() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("sequence_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Progress Sequence Test"),
            factory.create_string("Testing sequential progress updates"),
            80,
            factory.mock_materials_hash("sequence_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("sequence_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Update progress in sequence
        let progress_sequence = vec![10u32, 25u32, 50u32, 75u32, 90u32, 100u32];
        
        for (i, progress) in progress_sequence.iter().enumerate() {
            let result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), *progress);
            assert!(result.is_ok(), "Progress update {} should succeed", i + 1);
            
            let program = crate::storage::get_program(&env.env, &program_id).unwrap();
            let status = program.participants.get(farmer.clone()).unwrap();
            assert_eq!(status.progress, *progress, "Progress should be {} at step {}", progress, i + 1);
        }
        
        // Verify final state
        let final_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let final_status = final_program.participants.get(farmer).unwrap();
        assert_eq!(final_status.progress, 100, "Final progress should be 100%");
    }

    #[test]
    fn test_progress_update_regression() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("regression_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Progress Regression Test"),
            factory.create_string("Testing progress regression scenarios"),
            60,
            factory.mock_materials_hash("regression_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("regression_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Set initial progress
        update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 80).unwrap();
        
        // Regress progress (should be allowed - instructor might correct mistakes)
        let regressed_progress = 60u32;
        let result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), regressed_progress);
        assert!(result.is_ok(), "Progress regression should be allowed");
        
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let status = program.participants.get(farmer).unwrap();
        assert_eq!(status.progress, regressed_progress, "Progress should be regressed to {}", regressed_progress);
    }

    #[test]
    fn test_progress_update_multiple_farmers() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("multi_farmer_progress_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Multi-Farmer Progress Test"),
            factory.create_string("Testing progress updates for multiple farmers"),
            70,
            factory.mock_materials_hash("multi_farmer_progress_materials"),
        ).unwrap();
        
        // Enroll multiple farmers
        let farmers_and_progress = vec![
            (factory.mock_farmer("farmer_1"), 20u32),
            (factory.mock_farmer("farmer_2"), 45u32),
            (factory.mock_farmer("farmer_3"), 75u32),
            (factory.mock_farmer("farmer_4"), 90u32),
            (factory.mock_farmer("farmer_5"), 100u32),
        ];
        
        // Enroll all farmers
        for (farmer, _) in farmers_and_progress.iter() {
            enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        }
        
        // Update progress for each farmer
        for (farmer, expected_progress) in farmers_and_progress.iter() {
            let result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), *expected_progress);
            assert!(result.is_ok(), "Progress update should succeed for farmer");
        }
        
        // Verify all farmers have correct progress
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(program.participants.len(), farmers_and_progress.len());
        
        for (farmer, expected_progress) in farmers_and_progress.iter() {
            let status = program.participants.get(farmer.clone()).unwrap();
            assert_eq!(status.progress, *expected_progress, "Farmer should have progress {}", expected_progress);
        }
    }

    #[test]
    fn test_progress_update_preserves_other_fields() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("preserve_fields_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Preserve Fields Test"),
            factory.create_string("Testing that progress update preserves other fields"),
            40,
            factory.mock_materials_hash("preserve_fields_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("preserve_fields_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Get initial status
        let initial_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let initial_status = initial_program.participants.get(farmer.clone()).unwrap();
        let initial_farmer_id = initial_status.farmer_id.clone();
        let initial_certificate_id = initial_status.certificate_id.clone();
        
        // Update progress
        let new_progress = 65u32;
        update_progress(&env.env, instructor, program_id.clone(), farmer.clone(), new_progress).unwrap();
        
        // Verify other fields preserved
        let updated_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let updated_status = updated_program.participants.get(farmer).unwrap();
        
        assert_eq!(updated_status.farmer_id, initial_farmer_id, "Farmer ID should be preserved");
        assert_eq!(updated_status.certificate_id, initial_certificate_id, "Certificate ID should be preserved");
        assert_eq!(updated_status.progress, new_progress, "Progress should be updated");
    }
}

/// Test module for edge cases and error scenarios
mod participation_edge_cases {
    use super::*;

    #[test]
    fn test_enrollment_with_program_already_having_participants() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("existing_participants_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor,
            factory.create_string("Existing Participants Test"),
            factory.create_string("Testing enrollment when program already has participants"),
            45,
            factory.mock_materials_hash("existing_participants_materials"),
        ).unwrap();
        
        // Enroll first farmer
        let farmer1 = factory.mock_farmer("existing_farmer_1");
        let result1 = enroll_farmer(&env.env, farmer1.clone(), program_id.clone());
        assert!(result1.is_ok());
        
        // Enroll second farmer
        let farmer2 = factory.mock_farmer("existing_farmer_2");
        let result2 = enroll_farmer(&env.env, farmer2.clone(), program_id.clone());
        assert!(result2.is_ok(), "Should be able to enroll additional farmers");
        
        // Verify both farmers are enrolled
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(program.participants.len(), 2);
        assert!(program.participants.contains_key(farmer1));
        assert!(program.participants.contains_key(farmer2));
    }

    #[test]
    fn test_progress_update_nonexistent_program() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("nonexistent_program_instructor");
        let farmer = factory.mock_farmer("nonexistent_program_farmer");
        let nonexistent_program_id = factory.mock_program_id("nonexistent_program");
        
        let result = update_progress(&env.env, instructor, nonexistent_program_id, farmer, 50);
        
        TestAssertions::assert_contract_error(result, ContractError::ProgramNotFound);
    }

    #[test]
    fn test_progress_update_with_maximum_u32_value() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("max_u32_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Max U32 Test"),
            factory.create_string("Testing maximum u32 progress value"),
            30,
            factory.mock_materials_hash("max_u32_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("max_u32_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Try with maximum u32 value (should fail as it's > 100)
        let max_u32 = u32::MAX;
        let result = update_progress(&env.env, instructor, program_id, farmer, max_u32);
        
        TestAssertions::assert_contract_error(result, ContractError::InvalidData);
    }

    #[test]
    fn test_concurrent_enrollment_and_progress_update() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("concurrent_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Concurrent Operations Test"),
            factory.create_string("Testing concurrent enrollment and progress updates"),
            60,
            factory.mock_materials_hash("concurrent_materials"),
        ).unwrap();
        
        // Simulate concurrent operations by rapidly enrolling farmers and updating progress
        let farmers = vec![
            factory.mock_farmer("concurrent_farmer_1"),
            factory.mock_farmer("concurrent_farmer_2"),
            factory.mock_farmer("concurrent_farmer_3"),
        ];
        
        // Enroll all farmers first
        for farmer in farmers.iter() {
            let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
            assert!(result.is_ok(), "Concurrent enrollment should succeed");
        }
        
        // Update progress for all farmers
        for (i, farmer) in farmers.iter().enumerate() {
            let progress = ((i + 1) * 25) as u32; // 25%, 50%, 75%
            let result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), progress);
            assert!(result.is_ok(), "Concurrent progress update should succeed");
        }
        
        // Verify final state
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(program.participants.len(), 3);
        
        for (i, farmer) in farmers.iter().enumerate() {
            let status = program.participants.get(farmer.clone()).unwrap();
            let expected_progress = ((i + 1) * 25) as u32;
            assert_eq!(status.progress, expected_progress);
        }
    }

    #[test]
    fn test_enrollment_preserves_existing_participants() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("preserve_existing_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Preserve Existing Test"),
            factory.create_string("Testing that new enrollment preserves existing participants"),
            50,
            factory.mock_materials_hash("preserve_existing_materials"),
        ).unwrap();
        
        // Enroll first farmer and set progress
        let farmer1 = factory.mock_farmer("preserve_farmer_1");
        enroll_farmer(&env.env, farmer1.clone(), program_id.clone()).unwrap();
        update_progress(&env.env, instructor.clone(), program_id.clone(), farmer1.clone(), 40).unwrap();
        
        // Enroll second farmer
        let farmer2 = factory.mock_farmer("preserve_farmer_2");
        enroll_farmer(&env.env, farmer2.clone(), program_id.clone()).unwrap();
        
        // Verify first farmer's progress is preserved
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let farmer1_status = program.participants.get(farmer1).unwrap();
        let farmer2_status = program.participants.get(farmer2).unwrap();
        
        assert_eq!(farmer1_status.progress, 40, "First farmer's progress should be preserved");
        assert_eq!(farmer2_status.progress, 0, "Second farmer should start with 0 progress");
    }
}

/// Test module for performance and scalability scenarios
mod participation_performance {
    use super::*;

    #[test]
    fn test_bulk_farmer_enrollment_performance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let perf_helper = PerformanceTestHelper::new(&env);
        
        let instructor = factory.mock_instructor("bulk_enrollment_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor,
            factory.create_string("Bulk Enrollment Test"),
            factory.create_string("Testing bulk farmer enrollment performance"),
            40,
            factory.mock_materials_hash("bulk_enrollment_materials"),
        ).unwrap();
        
        // Generate many farmers for enrollment
        let farmer_count = 100;
        let farmers = perf_helper.generate_bulk_participants(farmer_count);
        
        let (enrolled_count, enrollment_time) = perf_helper.measure_execution_time(|| {
            let mut count = 0;
            
            for farmer in farmers.iter() {
                let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
                if result.is_ok() {
                    count += 1;
                }
            }
            
            count
        });
        
        assert_eq!(enrolled_count, farmer_count as usize, "All farmers should be enrolled");
        assert!(enrollment_time < 15000, "Bulk enrollment took too long: {} ms", enrollment_time);
        
        // Verify all farmers are actually enrolled
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(program.participants.len(), farmer_count as usize);
        
        // Spot check some farmers
        for i in [0, 25, 50, 75, 99].iter() {
            let farmer = &farmers[*i];
            let status = program.participants.get(farmer.clone()).unwrap();
            assert_eq!(status.farmer_id, *farmer);
            assert_eq!(status.progress, 0);
        }
    }

    #[test]
    fn test_bulk_progress_update_performance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let perf_helper = PerformanceTestHelper::new(&env);
        
        let instructor = factory.mock_instructor("bulk_progress_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Bulk Progress Test"),
            factory.create_string("Testing bulk progress update performance"),
            60,
            factory.mock_materials_hash("bulk_progress_materials"),
        ).unwrap();
        
        // Enroll farmers first
        let farmer_count = 50;
        let farmers = perf_helper.generate_bulk_participants(farmer_count);
        
        for farmer in farmers.iter() {
            enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        }
        
        // Update progress for all farmers
        let (updated_count, update_time) = perf_helper.measure_execution_time(|| {
            let mut count = 0;
            
            for (i, farmer) in farmers.iter().enumerate() {
                let progress = ((i % 5 + 1) * 20) as u32; // 20%, 40%, 60%, 80%, 100%
                let result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), progress);
                if result.is_ok() {
                    count += 1;
                }
            }
            
            count
        });
        
        assert_eq!(updated_count, farmer_count as usize, "All progress updates should succeed");
        assert!(update_time < 10000, "Bulk progress update took too long: {} ms", update_time);
        
        // Verify progress updates
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        for (i, farmer) in farmers.iter().enumerate() {
            let status = program.participants.get(farmer.clone()).unwrap();
            let expected_progress = ((i % 5 + 1) * 20) as u32;
            assert_eq!(status.progress, expected_progress, "Progress should be updated correctly for farmer {}", i);
        }
    }

    #[test]
    fn test_large_program_participation_scalability() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("scalability_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Scalability Test Program"),
            factory.create_string("Testing participation scalability with large number of farmers"),
            80,
            factory.mock_materials_hash("scalability_materials"),
        ).unwrap();
        
        // Enroll a large number of farmers
        let large_farmer_count = 200;
        let mut all_farmers = Vec::new();
        
        for i in 0..large_farmer_count {
            let farmer = factory.mock_farmer(&format!("scalability_farmer_{}", i));
            
            let result = enroll_farmer(&env.env, farmer.clone(), program_id.clone());
            assert!(result.is_ok(), "Farmer {} enrollment should succeed", i);
            
            all_farmers.push(farmer);
            
            // Periodically update some farmers' progress
            if i % 10 == 0 && i > 0 {
                let progress = ((i / 10) * 5).min(100) as u32;
                let update_result = update_progress(&env.env, instructor.clone(), program_id.clone(), all_farmers[i].clone(), progress);
                assert!(update_result.is_ok(), "Progress update should succeed for farmer {}", i);
            }
        }
        
        // Verify system state with large dataset
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(program.participants.len(), large_farmer_count);
        
        // Verify data integrity with random sampling
        for sample_idx in [0, 50, 100, 150, 199].iter() {
            let farmer = &all_farmers[*sample_idx];
            let status = program.participants.get(farmer.clone()).unwrap();
            assert_eq!(status.farmer_id, *farmer);
            
            // Check expected progress based on update pattern
            if sample_idx % 10 == 0 && *sample_idx > 0 {
                let expected_progress = ((sample_idx / 10) * 5).min(100) as u32;
                assert_eq!(status.progress, expected_progress, "Progress should match expected for farmer at index {}", sample_idx);
            } else {
                assert_eq!(status.progress, 0, "Non-updated farmer should have 0 progress at index {}", sample_idx);
            }
        }
    }
}

/// Test module for cross-module integration preparation
mod cross_module_integration {
    use super::*;

    #[test]
    fn test_participation_data_ready_for_certification() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("cert_ready_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Certification Ready Test"),
            factory.create_string("Testing participation data readiness for certification"),
            50,
            factory.mock_materials_hash("cert_ready_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("cert_ready_farmer");
        
        // Enroll farmer
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Progress to completion
        update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100).unwrap();
        
        // Verify data structure is ready for certification module
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let status = program.participants.get(farmer.clone()).unwrap();
        
        // Check all fields needed for certification
        assert_eq!(status.farmer_id, farmer, "Farmer ID needed for certificate recipient");
        assert_eq!(status.progress, 100, "100% progress needed for certificate eligibility");
        assert_eq!(status.certificate_id, BytesN::from_array(&env.env, &[0; 32]), "Certificate ID should be unissued for new certificate");
        assert_eq!(program.instructor_id, instructor, "Instructor ID needed for certificate authority");
        assert_eq!(program.program_id, program_id, "Program ID needed for certificate reference");
    }

    #[test]
    fn test_instructor_authorization_foundation_for_certification() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test that instructor authorization works properly for certification preparation
        let instructor = factory.mock_instructor("auth_foundation_instructor");
        let unauthorized_user = factory.mock_instructor("unauthorized_user");
        
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Auth Foundation Test"),
            factory.create_string("Testing instructor authorization foundation"),
            40,
            factory.mock_materials_hash("auth_foundation_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("auth_foundation_farmer");
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        
        // Authorized instructor should be able to update progress
        let auth_result = update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), 100);
        assert!(auth_result.is_ok(), "Authorized instructor should be able to update progress");
        
        // Unauthorized user should not be able to update progress
        let unauth_result = update_progress(&env.env, unauthorized_user, program_id.clone(), farmer.clone(), 50);
        TestAssertions::assert_contract_error(unauth_result, ContractError::NotInstructor);
        
        // This establishes that instructor authorization will work for certificate issuance
        let program = crate::storage::get_program(&env.env, &program_id).unwrap();
        assert_eq!(program.instructor_id, instructor, "Instructor should be properly identified for certificate authority");
    }

    #[test]
    fn test_multiple_programs_same_farmer_completion_tracking() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("multi_completion_instructor");
        let farmer = factory.mock_farmer("multi_completion_farmer");
        
        // Create multiple programs
        let programs_info = vec![
            ("Basic Agriculture", 30u32),
            ("Advanced Techniques", 50u32),
            ("Expert Certification", 80u32),
        ];
        
        let mut program_ids = Vec::new();
        
        for (title, duration) in programs_info.iter() {
            env.advance_time(1);
            
            let program_id = crate::training::create_training_program(
                &env.env,
                instructor.clone(),
                factory.create_string(title),
                factory.create_string(&format!("Description for {}", title)),
                *duration,
                factory.mock_materials_hash(&format!("{}_materials", title)),
            ).unwrap();
            
            program_ids.push(program_id);
        }
        
        // Enroll farmer in all programs
        for program_id in program_ids.iter() {
            enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        }
        
        // Complete programs at different rates
        let completion_rates = vec![100u32, 75u32, 50u32];
        
        for (i, program_id) in program_ids.iter().enumerate() {
            let progress = completion_rates[i];
            update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), progress).unwrap();
        }
        
        // Verify completion tracking across programs
        for (i, program_id) in program_ids.iter().enumerate() {
            let program = crate::storage::get_program(&env.env, program_id).unwrap();
            let status = program.participants.get(farmer.clone()).unwrap();
            
            assert_eq!(status.progress, completion_rates[i], "Progress should match for program {}", i);
            
            // Check if ready for certification (100% completion)
            if status.progress == 100 {
                assert_eq!(status.certificate_id, BytesN::from_array(&env.env, &[0; 32]), 
                          "Completed program should be ready for certification");
            }
        }
        
        // Verify that only the first program (100% complete) is ready for certification
        let completed_program = crate::storage::get_program(&env.env, &program_ids[0]).unwrap();
        let completed_status = completed_program.participants.get(farmer).unwrap();
        assert_eq!(completed_status.progress, 100, "First program should be completed and ready for certification");
    }

    #[test]
    fn test_participation_status_consistency_across_operations() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor = factory.mock_instructor("consistency_instructor");
        let program_id = crate::training::create_training_program(
            &env.env,
            instructor.clone(),
            factory.create_string("Consistency Test Program"),
            factory.create_string("Testing participation status consistency"),
            60,
            factory.mock_materials_hash("consistency_materials"),
        ).unwrap();
        
        let farmer = factory.mock_farmer("consistency_farmer");
        
        // Track status through various operations
        
        // 1. Initial enrollment
        enroll_farmer(&env.env, farmer.clone(), program_id.clone()).unwrap();
        let status_after_enrollment = {
            let program = crate::storage::get_program(&env.env, &program_id).unwrap();
            program.participants.get(farmer.clone()).unwrap()
        };
        
        // 2. Multiple progress updates
        let progress_updates = vec![25u32, 50u32, 75u32, 100u32];
        for progress in progress_updates.iter() {
            update_progress(&env.env, instructor.clone(), program_id.clone(), farmer.clone(), *progress).unwrap();
            
            let current_status = {
                let program = crate::storage::get_program(&env.env, &program_id).unwrap();
                program.participants.get(farmer.clone()).unwrap()
            };
            
            // Verify consistency of non-progress fields
            assert_eq!(current_status.farmer_id, status_after_enrollment.farmer_id, "Farmer ID should remain consistent");
            assert_eq!(current_status.certificate_id, status_after_enrollment.certificate_id, "Certificate ID should remain consistent until issued");
            assert_eq!(current_status.progress, *progress, "Progress should be updated correctly");
        }
        
        // 3. Final state verification for certification readiness
        let final_program = crate::storage::get_program(&env.env, &program_id).unwrap();
        let final_status = final_program.participants.get(farmer).unwrap();
        
        assert_eq!(final_status.farmer_id, farmer, "Final farmer ID should be correct");
        assert_eq!(final_status.progress, 100, "Final progress should be 100%");
        assert_eq!(final_status.certificate_id, BytesN::from_array(&env.env, &[0; 32]), "Certificate should still be unissued");
        
        // This verifies the data is ready and consistent for certificate issuance
    }
}