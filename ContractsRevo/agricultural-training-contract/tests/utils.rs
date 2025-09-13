#![cfg(test)]

use crate::utils::utils;
use crate::tests::utils::*;
use soroban_sdk::{BytesN, String, Vec, Address};

/// Test module for ID generation functionality
mod id_generation_tests {
    use super::*;

    #[test]
    fn test_generate_id_with_string_input() {
        let env = TestEnvironment::new();
        
        let input = String::from_str(&env.env, "test_program");
        let id = utils::generate_id(&env.env, input.clone());
        
        // Verify ID is 32 bytes
        assert_eq!(id.len(), 32);
        
        // Verify same input produces same ID
        let id2 = utils::generate_id(&env.env, input);
        assert_eq!(id, id2);
    }

    #[test]
    fn test_generate_id_with_program_tuple() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let title = factory.create_string("Advanced Crop Management");
        let instructor = factory.mock_instructor("instructor_alice");
        let timestamp = env.current_timestamp();
        
        let program_tuple = (title.clone(), instructor.clone(), timestamp);
        let id = utils::generate_id(&env.env, program_tuple);
        
        // Verify ID is 32 bytes
        assert_eq!(id.len(), 32);
        
        // Verify same tuple produces same ID
        let id2 = utils::generate_id(&env.env, (title, instructor, timestamp));
        assert_eq!(id, id2);
    }

    #[test]
    fn test_generate_id_with_certificate_tuple() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let program_id = factory.mock_program_id("test_program");
        let farmer_id = factory.mock_farmer("test_farmer");
        
        let certificate_tuple = (program_id.clone(), farmer_id.clone());
        let id = utils::generate_id(&env.env, certificate_tuple);
        
        // Verify ID is 32 bytes
        assert_eq!(id.len(), 32);
        
        // Verify same tuple produces same ID
        let id2 = utils::generate_id(&env.env, (program_id, farmer_id));
        assert_eq!(id, id2);
    }

    #[test]
    fn test_generate_id_uniqueness_different_inputs() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let input1 = factory.create_string("program_1");
        let input2 = factory.create_string("program_2");
        
        let id1 = utils::generate_id(&env.env, input1);
        let id2 = utils::generate_id(&env.env, input2);
        
        // Different inputs should produce different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_with_timestamp_variation() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let title = factory.create_string("Crop Management");
        let instructor = factory.mock_instructor("instructor");
        let timestamp1 = env.current_timestamp();
        
        let id1 = utils::generate_id(&env.env, (title.clone(), instructor.clone(), timestamp1));
        
        // Advance time
        env.advance_time(1);
        let timestamp2 = env.current_timestamp();
        
        let id2 = utils::generate_id(&env.env, (title, instructor, timestamp2));
        
        // Different timestamps should produce different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_with_address_variations() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let instructor1 = factory.mock_instructor("instructor_1");
        let instructor2 = factory.mock_instructor("instructor_2");
        let title = factory.create_string("Same Title");
        let timestamp = env.current_timestamp();
        
        let id1 = utils::generate_id(&env.env, (title.clone(), instructor1, timestamp));
        let id2 = utils::generate_id(&env.env, (title, instructor2, timestamp));
        
        // Different instructors should produce different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_deterministic_behavior() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let test_cases = vec![
            ("Basic Agriculture", 1000u64),
            ("Advanced Techniques", 2000u64),
            ("Soil Management", 3000u64),
            ("Pest Control", 4000u64),
        ];
        
        // Generate IDs multiple times for same inputs
        for (program_title, timestamp) in test_cases.iter() {
            let title = factory.create_string(program_title);
            let instructor = factory.mock_instructor("test_instructor");
            let input = (title.clone(), instructor.clone(), *timestamp);
            
            let id1 = utils::generate_id(&env.env, input.clone());
            let id2 = utils::generate_id(&env.env, input.clone());
            let id3 = utils::generate_id(&env.env, input);
            
            // All should be identical
            assert_eq!(id1, id2);
            assert_eq!(id2, id3);
            assert_eq!(id1, id3);
        }
    }

    #[test]
    fn test_generate_id_collision_resistance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let mut generated_ids = Vec::new(&env.env);
        let base_timestamp = env.current_timestamp();
        let instructor = factory.mock_instructor("collision_instructor");
        
        // Generate many IDs with similar but different inputs
        for i in 0..500 {
            let title = factory.create_string(&format!("Program_{}", i));
            let timestamp = base_timestamp + i as u64;
            let input = (title, instructor.clone(), timestamp);
            
            let id = utils::generate_id(&env.env, input);
            generated_ids.push_back(id);
        }
        
        // Verify all IDs are unique
        for i in 0..generated_ids.len() {
            for j in i + 1..generated_ids.len() {
                assert_ne!(
                    generated_ids.get(i).unwrap(),
                    generated_ids.get(j).unwrap(),
                    "Collision detected at positions {} and {}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_generate_id_with_complex_tuples() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test with complex nested data structures
        let instructor = factory.mock_instructor("complex_instructor");
        let farmer1 = factory.mock_farmer("farmer_1");
        let farmer2 = factory.mock_farmer("farmer_2");
        let title = factory.create_string("Complex Program");
        let timestamp = env.current_timestamp();
        let duration = 60u32;
        
        let complex_input = (
            instructor.clone(),
            title.clone(),
            duration,
            timestamp,
            farmer1.clone(),
        );
        
        let id = utils::generate_id(&env.env, complex_input);
        assert_eq!(id.len(), 32);
        
        // Verify slight change produces different ID
        let modified_input = (
            instructor,
            title,
            duration,
            timestamp,
            farmer2, // Different farmer
        );
        
        let id2 = utils::generate_id(&env.env, modified_input);
        assert_ne!(id, id2);
    }

    #[test]
    fn test_generate_id_with_empty_string() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let empty_string = factory.create_string("");
        let instructor = factory.mock_instructor("empty_test_instructor");
        let timestamp = env.current_timestamp();
        
        let input = (empty_string, instructor, timestamp);
        let id = utils::generate_id(&env.env, input);
        
        assert_eq!(id.len(), 32);
        
        // Should be different from non-empty inputs
        let non_empty = factory.create_string("non_empty");
        let instructor2 = factory.mock_instructor("empty_test_instructor");
        let id2 = utils::generate_id(&env.env, (non_empty, instructor2, timestamp));
        
        assert_ne!(id, id2);
    }

    #[test]
    fn test_generate_id_with_special_characters() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let special_inputs = vec![
            "program@domain.com",
            "program#hashtag",
            "program$price",
            "program%percent",
            "program&and",
            "program*star",
            "program+plus",
            "program=equals",
            "program[bracket]",
            "program{brace}",
            "program|pipe",
            "program\\backslash",
            "program:colon",
            "program;semicolon",
            "program\"quote",
            "program'apostrophe",
            "program<less>",
            "program,comma",
            "program.dot",
            "program?question",
            "program/slash",
            "program~tilde",
            "program`backtick",
        ];
        
        let instructor = factory.mock_instructor("special_char_instructor");
        let timestamp = env.current_timestamp();
        let mut special_ids = Vec::new(&env.env);
        
        for input_str in special_inputs.iter() {
            let input = factory.create_string(input_str);
            let id = utils::generate_id(&env.env, (input, instructor.clone(), timestamp));
            
            assert_eq!(id.len(), 32);
            special_ids.push_back(id);
        }
        
        // Verify all special character inputs produce unique IDs
        for i in 0..special_ids.len() {
            for j in i + 1..special_ids.len() {
                assert_ne!(
                    special_ids.get(i).unwrap(),
                    special_ids.get(j).unwrap(),
                    "Special character inputs '{}' and '{}' produced same ID",
                    special_inputs[i],
                    special_inputs[j]
                );
            }
        }
    }

    #[test]
    fn test_generate_id_with_unicode_inputs() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let unicode_inputs = vec![
            "农业培训",    // Chinese
            "التدريب الزراعي", // Arabic
            "Сельское хозяйство", // Russian
            "🌾🚜",        // Emojis
            "Café",        // Accented characters
            "Piña",        // Spanish with tilde
            "日本農業",     // Japanese
            "한국농업",     // Korean
        ];
        
        let instructor = factory.mock_instructor("unicode_instructor");
        let timestamp = env.current_timestamp();
        let mut unicode_ids = Vec::new(&env.env);
        
        for input_str in unicode_inputs.iter() {
            let input = factory.create_string(input_str);
            let id = utils::generate_id(&env.env, (input, instructor.clone(), timestamp));
            
            assert_eq!(id.len(), 32);
            unicode_ids.push_back(id);
        }
        
        // Verify Unicode inputs work correctly and produce unique IDs
        for i in 0..unicode_ids.len() {
            for j in i + 1..unicode_ids.len() {
                assert_ne!(
                    unicode_ids.get(i).unwrap(),
                    unicode_ids.get(j).unwrap(),
                    "Unicode inputs '{}' and '{}' produced same ID",
                    unicode_inputs[i],
                    unicode_inputs[j]
                );
            }
        }
    }
}

/// Test module for edge cases and boundary conditions
mod utils_edge_cases {
    use super::*;

    #[test]
    fn test_generate_id_with_very_long_input() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Create a very long string input
        let long_string = "A".repeat(5000);
        let input = factory.create_string(&long_string);
        let instructor = factory.mock_instructor("long_input_instructor");
        let timestamp = env.current_timestamp();
        
        let id = utils::generate_id(&env.env, (input, instructor, timestamp));
        
        // Should still produce 32-byte ID regardless of input length
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_generate_id_with_boundary_numeric_values() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let title = factory.create_string("Boundary Test");
        let instructor = factory.mock_instructor("boundary_instructor");
        
        let boundary_cases = vec![
            u64::MIN,
            u64::MAX,
            0u64,
            1u64,
            u64::MAX - 1,
        ];
        
        let mut boundary_ids = Vec::new(&env.env);
        
        for timestamp in boundary_cases.iter() {
            let input = (title.clone(), instructor.clone(), *timestamp);
            let id = utils::generate_id(&env.env, input);
            
            assert_eq!(id.len(), 32);
            boundary_ids.push_back(id);
        }
        
        // All boundary cases should produce unique IDs
        for i in 0..boundary_ids.len() {
            for j in i + 1..boundary_ids.len() {
                assert_ne!(
                    boundary_ids.get(i).unwrap(),
                    boundary_ids.get(j).unwrap(),
                    "Boundary case {} and {} produced same ID",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_generate_id_performance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let perf_helper = PerformanceTestHelper::new(&env);
        
        let instructor = factory.mock_instructor("performance_instructor");
        
        let (operations_count, execution_time) = perf_helper.measure_execution_time(|| {
            let mut count = 0;
            
            for i in 0..1000 {
                let title = factory.create_string(&format!("Performance Test Program {}", i));
                let timestamp = i as u64;
                let input = (title, instructor.clone(), timestamp);
                
                let _id = utils::generate_id(&env.env, input);
                count += 1;
            }
            
            count
        });
        
        assert_eq!(operations_count, 1000);
        assert!(execution_time < 5000, "ID generation performance too slow: {} ms for {} operations", execution_time, operations_count);
        
        // Calculate operations per second
        let ops_per_sec = (operations_count as f64 * 1000.0) / execution_time as f64;
        assert!(ops_per_sec > 100.0, "ID generation rate too slow: {:.2} ops/sec", ops_per_sec);
    }

    #[test]
    fn test_generate_id_memory_efficiency() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Generate many IDs and verify they don't interfere with each other
        let mut all_ids = Vec::new(&env.env);
        let instructor = factory.mock_instructor("memory_instructor");
        
        for batch in 0..10 {
            let mut batch_ids = Vec::new(&env.env);
            
            for i in 0..100 {
                let title = factory.create_string(&format!("Memory Test Batch {} Program {}", batch, i));
                let timestamp = (batch * 100 + i) as u64;
                let input = (title, instructor.clone(), timestamp);
                
                let id = utils::generate_id(&env.env, input);
                batch_ids.push_back(id);
            }
            
            // Verify batch consistency
            assert_eq!(batch_ids.len(), 100);
            
            // Add to global collection
            for id in batch_ids.iter() {
                all_ids.push_back(*id);
            }
        }
        
        // Verify total collection
        assert_eq!(all_ids.len(), 1000);
        
        // Spot check uniqueness across batches
        for i in 0..all_ids.len() {
            for j in (i + 1)..std::cmp::min(i + 50, all_ids.len()) {
                assert_ne!(
                    all_ids.get(i).unwrap(),
                    all_ids.get(j).unwrap(),
                    "Memory efficiency test found duplicate IDs at positions {} and {}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_generate_id_consistency_across_operations() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Simulate multiple contract operations using ID generation
        let base_inputs = vec![
            ("Crop Rotation Program", 1000u64),
            ("Pest Management Course", 2000u64),
            ("Soil Health Training", 3000u64),
            ("Organic Certification", 4000u64),
        ];
        
        let instructor = factory.mock_instructor("consistency_instructor");
        let mut first_pass_ids = Vec::new(&env.env);
        let mut second_pass_ids = Vec::new(&env.env);
        
        // First pass - generate IDs
        for (title_str, timestamp) in base_inputs.iter() {
            let title = factory.create_string(title_str);
            let input = (title, instructor.clone(), *timestamp);
            let id = utils::generate_id(&env.env, input);
            first_pass_ids.push_back(id);
        }
        
        // Second pass - generate same IDs again
        for (title_str, timestamp) in base_inputs.iter() {
            let title = factory.create_string(title_str);
            let input = (title, instructor.clone(), *timestamp);
            let id = utils::generate_id(&env.env, input);
            second_pass_ids.push_back(id);
        }
        
        // IDs should be identical across passes
        assert_eq!(first_pass_ids.len(), second_pass_ids.len());
        for i in 0..first_pass_ids.len() {
            assert_eq!(
                first_pass_ids.get(i).unwrap(),
                second_pass_ids.get(i).unwrap(),
                "ID inconsistency at position {}",
                i
            );
        }
    }
}

/// Test module for integration with agricultural training contract functionality
mod training_contract_integration {
    use super::*;

    #[test]
    fn test_id_generation_for_program_creation() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test ID generation in the context of program creation
        let title = factory.create_string("Integration Test Program");
        let instructor = factory.mock_instructor("integration_instructor");
        let timestamp = env.current_timestamp();
        
        // This mimics how training.rs uses the utility
        let program_id = utils::generate_id(&env.env, (title.clone(), instructor.clone(), timestamp));
        
        // Verify the ID works for program operations
        assert_eq!(program_id.len(), 32);
        
        // Verify ID generation matches training module expectations
        let expected_id = crate::utils::utils::generate_id(&env.env, (title, instructor, timestamp));
        assert_eq!(program_id, expected_id);
    }

    #[test]
    fn test_id_generation_for_certificate_issuance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test ID generation for certificate scenarios
        let program_id = factory.mock_program_id("certificate_test_program");
        let farmer_id = factory.mock_farmer("certificate_test_farmer");
        
        // Generate certificate ID like the certification module would
        let certificate_id = utils::generate_id(&env.env, (program_id.clone(), farmer_id.clone()));
        
        assert_eq!(certificate_id.len(), 32);
        
        // Verify different combinations produce different IDs
        let different_farmer = factory.mock_farmer("different_farmer");
        let certificate_id2 = utils::generate_id(&env.env, (program_id, different_farmer));
        
        assert_ne!(certificate_id, certificate_id2);
    }

    #[test]
    fn test_id_generation_real_world_training_scenarios() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Simulate real-world agricultural training scenarios
        let scenarios = vec![
            // Program creation scenarios
            ("Sustainable Farming Practices Level 1", "expert_instructor_1", 1000u64),
            ("Advanced Crop Rotation Techniques", "senior_instructor_2", 2000u64),
            ("Integrated Pest Management Certification", "specialist_instructor_3", 3000u64),
            ("Soil Health and Nutrition Analysis", "research_instructor_4", 4000u64),
            ("Precision Agriculture Technology Training", "tech_instructor_5", 5000u64),
            ("Organic Certification Preparation Course", "organic_specialist_6", 6000u64),
        ];
        
        let mut scenario_ids = Vec::new(&env.env);
        
        for (program_title, instructor_name, timestamp) in scenarios.iter() {
            let title = factory.create_string(program_title);
            let instructor = factory.mock_instructor(instructor_name);
            let input = (title, instructor, *timestamp);
            
            let id = utils::generate_id(&env.env, input);
            
            assert_eq!(id.len(), 32);
            scenario_ids.push_back(id);
        }
        
        // Verify all real-world scenarios produce unique IDs
        for i in 0..scenario_ids.len() {
            for j in i + 1..scenario_ids.len() {
                assert_ne!(
                    scenario_ids.get(i).unwrap(),
                    scenario_ids.get(j).unwrap(),
                    "Real-world scenario '{}' and '{}' produced duplicate IDs",
                    scenarios[i].0,
                    scenarios[j].0
                );
            }
        }
    }

    #[test]
    fn test_id_generation_cross_module_consistency() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test that ID generation is consistent across different modules
        let instructor = factory.mock_instructor("cross_module_instructor");
        let title = factory.create_string("Cross Module Test Program");
        let timestamp = env.current_timestamp();
        let farmer = factory.mock_farmer("cross_module_farmer");
        
        // Program ID (training module)
        let program_id = utils::generate_id(&env.env, (title, instructor, timestamp));
        
        // Certificate ID (certification module)  
        let certificate_id = utils::generate_id(&env.env, (program_id.clone(), farmer));
        
        // Verify IDs are different but deterministic
        assert_ne!(program_id, certificate_id);
        assert_eq!(program_id.len(), 32);
        assert_eq!(certificate_id.len(), 32);
        
        // Verify reproducibility
        let program_id2 = utils::generate_id(&env.env, (
            factory.create_string("Cross Module Test Program"),
            factory.mock_instructor("cross_module_instructor"),
            timestamp
        ));
        
        assert_eq!(program_id, program_id2, "Program ID should be reproducible");
    }
}

#[cfg(test)]
mod utility_module_tests {
    use super::*;

    #[test]
    fn test_utility_module_accessibility() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Verify the utility module is accessible and functional
        let test_input = factory.create_string("accessibility_test");
        let result = utils::generate_id(&env.env, test_input);
        
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_utility_function_signature_flexibility() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test that the function accepts various input types
        
        // String input
        let string_input = factory.create_string("test");
        let _id1 = utils::generate_id(&env.env, string_input);
        
        // Address input
        let address_input = factory.mock_instructor("test_instructor");
        let _id2 = utils::generate_id(&env.env, address_input);
        
        // Tuple input
        let tuple_input = (123u64, 456u32);
        let _id3 = utils::generate_id(&env.env, tuple_input);
        
        // Complex tuple input
        let complex_input = (
            factory.create_string("complex"),
            factory.mock_instructor("instructor"),
            123u64,
        );
        let _id4 = utils::generate_id(&env.env, complex_input);
        
        // All should work without compilation errors
    }

    #[test]
    fn test_deterministic_properties_for_contract_reliability() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test deterministic properties crucial for contract reliability
        let instructor = factory.mock_instructor("reliability_instructor");
        let title = factory.create_string("Reliability Test Program");
        let timestamp = 12345u64;
        
        // Generate ID multiple times
        let ids = vec![
            utils::generate_id(&env.env, (title.clone(), instructor.clone(), timestamp)),
            utils::generate_id(&env.env, (title.clone(), instructor.clone(), timestamp)),
            utils::generate_id(&env.env, (title.clone(), instructor.clone(), timestamp)),
            utils::generate_id(&env.env, (title.clone(), instructor.clone(), timestamp)),
            utils::generate_id(&env.env, (title, instructor, timestamp)),
        ];
        
        // All IDs should be identical (deterministic)
        for i in 1..ids.len() {
            assert_eq!(ids[0], ids[i], "ID generation should be deterministic for contract reliability");
        }
    
    }
}