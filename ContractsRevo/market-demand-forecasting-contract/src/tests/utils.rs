#![cfg(test)]

use crate::utils::utils;
use crate::tests::utils::*;
use soroban_sdk::{BytesN, String, Vec};

/// Test module for ID generation functionality
mod id_generation_tests {
    use super::*;

    #[test]
    fn test_generate_id_with_string_input() {
        let env = TestEnvironment::new();
        
        let input = String::from_str(&env.env, "test_string");
        let id = utils::generate_id(&env.env, input.clone());
        
        // Verify ID is 32 bytes
        assert_eq!(id.len(), 32);
        
        // Verify same input produces same ID
        let id2 = utils::generate_id(&env.env, input);
        assert_eq!(id, id2);
    }

    #[test]
    fn test_generate_id_with_tuple_input() {
        let env = TestEnvironment::new();
        
        let name = String::from_str(&env.env, "product_name");
        let timestamp = env.current_timestamp();
        let input_tuple = (name.clone(), timestamp);
        
        let id = utils::generate_id(&env.env, input_tuple);
        
        // Verify ID is 32 bytes
        assert_eq!(id.len(), 32);
        
        // Verify same tuple produces same ID
        let id2 = utils::generate_id(&env.env, (name, timestamp));
        assert_eq!(id, id2);
    }

    #[test]
    fn test_generate_id_uniqueness_with_different_inputs() {
        let env = TestEnvironment::new();
        
        let input1 = String::from_str(&env.env, "input1");
        let input2 = String::from_str(&env.env, "input2");
        
        let id1 = utils::generate_id(&env.env, input1);
        let id2 = utils::generate_id(&env.env, input2);
        
        // Different inputs should produce different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_with_timestamp_variation() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "wheat");
        let timestamp1 = env.current_timestamp();
        
        let id1 = utils::generate_id(&env.env, (product_name.clone(), timestamp1));
        
        // Advance time
        env.advance_time(1);
        let timestamp2 = env.current_timestamp();
        
        let id2 = utils::generate_id(&env.env, (product_name, timestamp2));
        
        // Different timestamps should produce different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_with_complex_tuple() {
        let env = TestEnvironment::new();
        
        let oracle_id = TestDataFactory::new(&env.env).mock_data_hash("oracle");
        let product_id = TestDataFactory::new(&env.env).mock_product_id("product");
        let region = String::from_str(&env.env, "US-West");
        let timestamp = env.current_timestamp();
        
        let complex_input = (oracle_id.clone(), product_id.clone(), region.clone(), timestamp);
        let id = utils::generate_id(&env.env, complex_input);
        
        // Verify ID generation works with complex inputs
        assert_eq!(id.len(), 32);
        
        // Verify reproducibility
        let id2 = utils::generate_id(&env.env, (oracle_id, product_id, region, timestamp));
        assert_eq!(id, id2);
    }

    #[test]
    fn test_generate_id_with_numeric_inputs() {
        let env = TestEnvironment::new();
        
        let demand = 1500i128;
        let confidence = 85u32;
        let timestamp = env.current_timestamp();
        
        let numeric_input = (demand, confidence, timestamp);
        let id = utils::generate_id(&env.env, numeric_input);
        
        assert_eq!(id.len(), 32);
        
        // Verify slight changes produce different IDs
        let id2 = utils::generate_id(&env.env, (demand + 1, confidence, timestamp));
        assert_ne!(id, id2);
        
        let id3 = utils::generate_id(&env.env, (demand, confidence + 1, timestamp));
        assert_ne!(id, id3);
    }

    #[test]
    fn test_generate_id_deterministic_behavior() {
        let env = TestEnvironment::new();
        
        let test_cases = vec![
            ("wheat", 1000u64),
            ("corn", 2000u64),
            ("soybeans", 3000u64),
            ("rice", 4000u64),
            ("barley", 5000u64),
        ];
        
        // Generate IDs multiple times for same inputs
        for (product_name, timestamp) in test_cases.iter() {
            let name = String::from_str(&env.env, product_name);
            let input = (name.clone(), *timestamp);
            
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
        
        let mut generated_ids = Vec::new(&env.env);
        let base_timestamp = env.current_timestamp();
        
        // Generate many IDs with similar but different inputs
        for i in 0..1000 {
            let name = String::from_str(&env.env, &format!("product_{}", i));
            let timestamp = base_timestamp + i as u64;
            let input = (name, timestamp);
            
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
    fn test_generate_id_with_empty_string() {
        let env = TestEnvironment::new();
        
        let empty_string = String::from_str(&env.env, "");
        let id = utils::generate_id(&env.env, empty_string);
        
        assert_eq!(id.len(), 32);
        
        // Should be different from other inputs
        let non_empty = String::from_str(&env.env, "non_empty");
        let id2 = utils::generate_id(&env.env, non_empty);
        
        assert_ne!(id, id2);
    }

    #[test]
    fn test_generate_id_with_special_characters() {
        let env = TestEnvironment::new();
        
        let special_inputs = vec![
            "product@domain.com",
            "product#hashtag",
            "product$price",
            "product%percent",
            "product&and",
            "product*star",
            "product+plus",
            "product=equals",
            "product[bracket]",
            "product{brace}",
            "product|pipe",
            "product\\backslash",
            "product:colon",
            "product;semicolon",
            "product\"quote",
            "product'apostrophe",
            "product<less>",
            "product,comma",
            "product.dot",
            "product?question",
            "product/slash",
            "product~tilde",
            "product`backtick",
        ];
        
        let mut special_ids = Vec::new(&env.env);
        
        for input_str in special_inputs.iter() {
            let input = String::from_str(&env.env, input_str);
            let id = utils::generate_id(&env.env, input);
            
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
        
        let unicode_inputs = vec![
            "小麦",      // Chinese
            "القمح",     // Arabic
            "пшеница",   // Russian
            "🌾",        // Emoji
            "café",      // Accented
            "piña",      // Spanish with tilde
        ];
        
        let mut unicode_ids = Vec::new(&env.env);
        
        for input_str in unicode_inputs.iter() {
            let input = String::from_str(&env.env, input_str);
            let id = utils::generate_id(&env.env, input);
            
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
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_generate_id_with_very_long_input() {
        let env = TestEnvironment::new();
        
        // Create a very long string input
        let long_string = "a".repeat(10000);
        let input = String::from_str(&env.env, &long_string);
        let id = utils::generate_id(&env.env, input);
        
        // Should still produce 32-byte ID regardless of input length
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_generate_id_with_boundary_numeric_values() {
        let env = TestEnvironment::new();
        
        let boundary_cases = vec![
            (i128::MIN, u32::MIN, u64::MIN),
            (i128::MAX, u32::MAX, u64::MAX),
            (0i128, 0u32, 0u64),
            (1i128, 1u32, 1u64),
            (-1i128, u32::MAX - 1, u64::MAX - 1),
        ];
        
        let mut boundary_ids = Vec::new(&env.env);
        
        for (i128_val, u32_val, u64_val) in boundary_cases.iter() {
            let input = (*i128_val, *u32_val, *u64_val);
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
        let perf_helper = PerformanceTestHelper::new(&env);
        
        let (operations_count, execution_time) = perf_helper.measure_execution_time(|| {
            let mut count = 0;
            
            for i in 0..1000 {
                let input = (
                    String::from_str(&env.env, &format!("perf_test_{}", i)),
                    i as u64,
                    i as i128,
                );
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
        
        // Generate many IDs and verify they don't interfere with each other
        let mut all_ids = Vec::new(&env.env);
        
        for batch in 0..10 {
            let mut batch_ids = Vec::new(&env.env);
            
            for i in 0..100 {
                let input = (
                    String::from_str(&env.env, &format!("batch_{}_{}", batch, i)),
                    (batch * 100 + i) as u64,
                );
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
}

/// Test module for integration with contract functionality
mod integration_tests {
    use super::*;

    #[test]
    fn test_id_generation_for_product_registration() {
        let env = TestEnvironment::new();
        
        // Test ID generation in the context of product registration
        let product_name = String::from_str(&env.env, "integration_wheat");
        let timestamp = env.current_timestamp();
        
        // This mimics how data.rs uses the utility
        let product_id = utils::generate_id(&env.env, (product_name.clone(), timestamp));
        
        // Verify the ID works for product operations
        assert_eq!(product_id.len(), 32);
        
        // Simulate product storage (this would be done by the storage module)
        // The ID should be usable as a key
        let key_test = format!("product_{}", hex::encode(product_id.to_array()));
        assert!(key_test.starts_with("product_"));
        assert_eq!(key_test.len(), "product_".len() + 64); // 64 hex chars for 32 bytes
    }

    #[test]
    fn test_id_generation_for_forecast_ids() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test ID generation for forecast scenarios
        let oracle_id = factory.mock_data_hash("oracle_address");
        let product_id = factory.mock_product_id("test_product");
        let region = String::from_str(&env.env, "test_region");
        let timestamp = env.current_timestamp();
        
        // Generate forecast ID like the forecasting module would
        let forecast_id = utils::generate_id(&env.env, (
            oracle_id.clone(),
            product_id.clone(),
            region.clone(),
            timestamp,
        ));
        
        assert_eq!(forecast_id.len(), 32);
        
        // Verify different combinations produce different IDs
        env.advance_time(1);
        let forecast_id2 = utils::generate_id(&env.env, (
            oracle_id,
            product_id,
            region,
            env.current_timestamp(),
        ));
        
        assert_ne!(forecast_id, forecast_id2);
    }

    #[test]
    fn test_id_generation_consistency_across_operations() {
        let env = TestEnvironment::new();
        
        // Simulate multiple contract operations using ID generation
        let base_inputs = vec![
            ("product_wheat", 1000u64),
            ("product_corn", 2000u64),
            ("product_soybeans", 3000u64),
        ];
        
        let mut first_pass_ids = Vec::new(&env.env);
        let mut second_pass_ids = Vec::new(&env.env);
        
        // First pass - generate IDs
        for (name, timestamp) in base_inputs.iter() {
            let input = (String::from_str(&env.env, name), *timestamp);
            let id = utils::generate_id(&env.env, input);
            first_pass_ids.push_back(id);
        }
        
        // Second pass - generate same IDs again
        for (name, timestamp) in base_inputs.iter() {
            let input = (String::from_str(&env.env, name), *timestamp);
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

    #[test]
    fn test_id_generation_with_real_world_scenarios() {
        let env = TestEnvironment::new();
        
        // Simulate real-world contract usage patterns
        let scenarios = vec![
            // Product registration scenarios
            ("premium_wheat_variety_A", env.current_timestamp()),
            ("organic_corn_hybrid_B2", env.current_timestamp() + 1),
            ("non_gmo_soybeans_grade_1", env.current_timestamp() + 2),
            
            // Oracle scenarios
            ("weather_oracle_primary", env.current_timestamp() + 100),
            ("market_oracle_secondary", env.current_timestamp() + 101),
            ("satellite_data_oracle", env.current_timestamp() + 102),
            
            // Regional scenarios
            ("forecast_us_midwest_corn", env.current_timestamp() + 1000),
            ("forecast_eu_plains_wheat", env.current_timestamp() + 1001),
            ("forecast_asia_rice_premium", env.current_timestamp() + 1002),
        ];
        
        let mut scenario_ids = Vec::new(&env.env);
        
        for (scenario_name, timestamp) in scenarios.iter() {
            let input = (String::from_str(&env.env, scenario_name), *timestamp);
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
}

#[cfg(test)]
mod utility_module_tests {
    use super::*;

    #[test]
    fn test_utility_module_accessibility() {
        let env = TestEnvironment::new();
        
        // Verify the utility module is accessible and functional
        let test_input = String::from_str(&env.env, "accessibility_test");
        let result = utils::generate_id(&env.env, test_input);
        
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_utility_function_signature() {
        let env = TestEnvironment::new();
        
        // Test that the function accepts various input types
        
        // String input
        let string_input = String::from_str(&env.env, "test");
        let _id1 = utils::generate_id(&env.env, string_input);
        
        // Tuple input
        let tuple_input = (123i128, 456u32);
        let _id2 = utils::generate_id(&env.env, tuple_input);
        
        // Complex tuple input
        let complex_input = (
            String::from_str(&env.env, "complex"),
            123u64,
            456i128,
        );
        let _id3 = utils::generate_id(&env.env, complex_input);
        
        // All should work without compilation errors
    }
}