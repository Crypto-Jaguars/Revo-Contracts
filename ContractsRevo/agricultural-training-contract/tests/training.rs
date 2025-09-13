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
}