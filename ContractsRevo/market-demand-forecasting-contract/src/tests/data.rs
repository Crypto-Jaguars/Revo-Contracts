#![cfg(test)]

use crate::{
    error::ContractError,
    storage::Product,
    data::register_product,
    tests::utils::*,
};
use soroban_sdk::{BytesN, String, Vec};

/// Test module for basic product registration functionality
mod product_registration {
    use super::*;

    #[test]
    fn test_successful_product_registration() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let product_name = String::from_str(&env.env, "wheat");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(1000);
        historical_demand.push_back(1200);
        historical_demand.push_back(1100);
        
        let result = register_product(&env.env, product_name.clone(), historical_demand.clone());
        
        assert!(result.is_ok());
        let product_id = result.unwrap();
        
        // Verify product was stored correctly
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.product_id, product_id);
        assert_eq!(stored_product.name, product_name);
        assert_eq!(stored_product.historical_demand, historical_demand);
        
        // Verify product ID was added to global list
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert!(all_product_ids.contains(&product_id));
    }

    #[test]
    fn test_product_registration_with_empty_name() {
        let env = TestEnvironment::new();
        
        let empty_name = String::from_str(&env.env, "");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(500);
        
        let result = register_product(&env.env, empty_name, historical_demand);
        
        TestAssertions::assert_contract_error(result, ContractError::InvalidData);
    }

    #[test]
    fn test_product_registration_with_empty_historical_data() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "corn");
        let empty_historical_demand = Vec::new(&env.env);
        
        let result = register_product(&env.env, product_name.clone(), empty_historical_demand.clone());
        
        // Should succeed even with empty historical data
        assert!(result.is_ok());
        let product_id = result.unwrap();
        
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.name, product_name);
        assert_eq!(stored_product.historical_demand.len(), 0);
    }

    #[test]
    fn test_product_registration_generates_unique_ids() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "soybeans");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(800);
        
        // Register the same product twice
        let product_id1 = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        // Advance time to ensure different timestamp
        env.advance_time(1);
        
        let product_id2 = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        // IDs should be different due to different timestamps
        assert_ne!(product_id1, product_id2);
        
        // Both products should exist
        let product1 = crate::storage::get_product(&env.env, &product_id1).unwrap();
        let product2 = crate::storage::get_product(&env.env, &product_id2).unwrap();
        
        assert_eq!(product1.name, product_name);
        assert_eq!(product2.name, product_name);
        assert_ne!(product1.product_id, product2.product_id);
    }

    #[test]
    fn test_product_registration_with_various_name_lengths() {
        let env = TestEnvironment::new();
        
        let test_names = vec![
            "a",                    // Single character
            "rice",                 // Short name
            "agricultural_product", // Long name
            "product-with-dashes",  // With special characters
            "product_with_underscores",
            "Product With Spaces",  // With spaces
        ];
        
        let mut registered_ids = Vec::new(&env.env);
        
        for name in test_names.iter() {
            let product_name = String::from_str(&env.env, name);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(1000);
            
            env.advance_time(1); // Ensure unique timestamps
            
            let result = register_product(&env.env, product_name.clone(), historical_demand);
            assert!(result.is_ok(), "Failed to register product with name: {}", name);
            
            let product_id = result.unwrap();
            registered_ids.push_back(product_id.clone());
            
            // Verify the product was stored correctly
            let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
            assert_eq!(stored_product.name.to_string(), *name);
        }
        
        // Verify all IDs are unique
        for i in 0..registered_ids.len() {
            for j in i + 1..registered_ids.len() {
                assert_ne!(registered_ids.get(i).unwrap(), registered_ids.get(j).unwrap());
            }
        }
    }
}

/// Test module for historical demand data validation and management
mod historical_demand_management {
    use super::*;

    #[test]
    fn test_product_registration_with_single_historical_point() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "barley");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(750);
        
        let product_id = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.historical_demand.len(), 1);
        assert_eq!(stored_product.historical_demand.get(0).unwrap(), 750);
    }

    #[test]
    fn test_product_registration_with_extensive_historical_data() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "cotton");
        let mut historical_demand = Vec::new(&env.env);
        
        // Add 12 months of historical data
        let monthly_demands = vec![
            800, 850, 900, 950, 1000, 1100,  // Jan-Jun
            1200, 1150, 1050, 950, 900, 850  // Jul-Dec
        ];
        
        for demand in monthly_demands {
            historical_demand.push_back(demand);
        }
        
        let product_id = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.historical_demand.len(), 12);
        
        // Verify all historical data is preserved
        for i in 0..12 {
            assert_eq!(
                stored_product.historical_demand.get(i).unwrap(),
                monthly_demands[i]
            );
        }
    }

    #[test]
    fn test_product_registration_with_negative_historical_demand() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "test_negative");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(-100); // Negative demand
        historical_demand.push_back(500);  // Positive demand
        
        // Should succeed - negative values might represent surpluses or corrections
        let result = register_product(&env.env, product_name.clone(), historical_demand.clone());
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.historical_demand.get(0).unwrap(), -100);
        assert_eq!(stored_product.historical_demand.get(1).unwrap(), 500);
    }

    #[test]
    fn test_product_registration_with_zero_historical_demand() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "test_zero");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(0);    // Zero demand
        historical_demand.push_back(100);  // Positive demand
        historical_demand.push_back(0);    // Zero demand again
        
        let result = register_product(&env.env, product_name.clone(), historical_demand.clone());
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.historical_demand.len(), 3);
        assert_eq!(stored_product.historical_demand.get(0).unwrap(), 0);
        assert_eq!(stored_product.historical_demand.get(2).unwrap(), 0);
    }

    #[test]
    fn test_product_registration_with_extreme_historical_values() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "extreme_values");
        let mut historical_demand = Vec::new(&env.env);
        
        // Test boundary values
        historical_demand.push_back(i128::MIN);
        historical_demand.push_back(i128::MAX);
        historical_demand.push_back(0);
        historical_demand.push_back(1);
        historical_demand.push_back(-1);
        
        let result = register_product(&env.env, product_name.clone(), historical_demand.clone());
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        
        assert_eq!(stored_product.historical_demand.get(0).unwrap(), i128::MIN);
        assert_eq!(stored_product.historical_demand.get(1).unwrap(), i128::MAX);
        assert_eq!(stored_product.historical_demand.get(2).unwrap(), 0);
        assert_eq!(stored_product.historical_demand.get(3).unwrap(), 1);
        assert_eq!(stored_product.historical_demand.get(4).unwrap(), -1);
    }
}

/// Test module for duplicate product registration scenarios
mod duplicate_registration_scenarios {
    use super::*;

    #[test]
    fn test_duplicate_product_name_registration() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "duplicate_test");
        let mut historical_demand1 = Vec::new(&env.env);
        historical_demand1.push_back(1000);
        
        let mut historical_demand2 = Vec::new(&env.env);
        historical_demand2.push_back(1500);
        
        // Register first product
        let product_id1 = register_product(&env.env, product_name.clone(), historical_demand1.clone()).unwrap();
        
        // Advance time to ensure different ID generation
        env.advance_time(10);
        
        // Register product with same name but different historical data
        let product_id2 = register_product(&env.env, product_name.clone(), historical_demand2.clone()).unwrap();
        
        // Both registrations should succeed with different IDs
        assert_ne!(product_id1, product_id2);
        
        // Verify both products exist independently
        let product1 = crate::storage::get_product(&env.env, &product_id1).unwrap();
        let product2 = crate::storage::get_product(&env.env, &product_id2).unwrap();
        
        assert_eq!(product1.name, product_name);
        assert_eq!(product2.name, product_name);
        assert_eq!(product1.historical_demand.get(0).unwrap(), 1000);
        assert_eq!(product2.historical_demand.get(0).unwrap(), 1500);
        
        // Verify both are in the global product list
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert!(all_product_ids.contains(&product_id1));
        assert!(all_product_ids.contains(&product_id2));
    }

    #[test]
    fn test_exact_duplicate_registration_attempt() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "exact_duplicate");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(900);
        historical_demand.push_back(950);
        
        // Register first product
        let product_id1 = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        // Try to register exact same product immediately (same timestamp)
        let product_id2 = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        // Due to timestamp being part of ID generation, these should typically be different
        // unless the system clock hasn't advanced, but the function should still succeed
        let product1 = crate::storage::get_product(&env.env, &product_id1).unwrap();
        let product2 = crate::storage::get_product(&env.env, &product_id2).unwrap();
        
        assert_eq!(product1.name, product2.name);
        assert_eq!(product1.historical_demand, product2.historical_demand);
    }

    #[test]
    fn test_case_sensitive_product_names() {
        let env = TestEnvironment::new();
        
        let variations = vec![
            "wheat",
            "WHEAT", 
            "Wheat",
            "WhEaT",
        ];
        
        let mut registered_ids = Vec::new(&env.env);
        
        for variation in variations.iter() {
            let product_name = String::from_str(&env.env, variation);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(1000);
            
            env.advance_time(1);
            
            let product_id = register_product(&env.env, product_name.clone(), historical_demand).unwrap();
            registered_ids.push_back(product_id.clone());
            
            let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
            assert_eq!(stored_product.name.to_string(), *variation);
        }
        
        // All should be treated as different products
        assert_eq!(registered_ids.len(), 4);
        
        // Verify all IDs are unique
        for i in 0..registered_ids.len() {
            for j in i + 1..registered_ids.len() {
                assert_ne!(registered_ids.get(i).unwrap(), registered_ids.get(j).unwrap());
            }
        }
    }
}

/// Test module for product retrieval and validation after registration
mod product_retrieval_validation {
    use super::*;

    #[test]
    fn test_retrieve_nonexistent_product() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let nonexistent_id = factory.mock_product_id("nonexistent");
        let result = crate::storage::get_product(&env.env, &nonexistent_id);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_product_metadata_integrity() {
        let env = TestEnvironment::new();
        
        let original_timestamp = env.current_timestamp();
        let product_name = String::from_str(&env.env, "integrity_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(1300);
        historical_demand.push_back(1400);
        
        let product_id = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        
        // Verify all metadata is preserved
        assert_eq!(stored_product.product_id, product_id);
        assert_eq!(stored_product.name, product_name);
        assert_eq!(stored_product.historical_demand, historical_demand);
        
        // Verify ID generation includes timestamp (ID should be different from a simple hash of name)
        let simple_name_hash = env.env.crypto().sha256(&product_name);
        assert_ne!(product_id, simple_name_hash);
    }

    #[test]
    fn test_product_list_consistency() {
        let env = TestEnvironment::new();
        
        let products = vec![
            ("product1", vec![100, 200]),
            ("product2", vec![300, 400, 500]),
            ("product3", vec![600]),
        ];
        
        let mut registered_ids = Vec::new(&env.env);
        
        for (name, demands) in products.iter() {
            let product_name = String::from_str(&env.env, name);
            let mut historical_demand = Vec::new(&env.env);
            
            for &demand in demands {
                historical_demand.push_back(demand);
            }
            
            env.advance_time(1);
            
            let product_id = register_product(&env.env, product_name, historical_demand).unwrap();
            registered_ids.push_back(product_id);
        }
        
        // Verify all products are in the global list
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert_eq!(all_product_ids.len(), 3);
        
        for registered_id in registered_ids.iter() {
            assert!(all_product_ids.contains(registered_id));
        }
        
        // Verify each product can be retrieved correctly
        for (i, (name, demands)) in products.iter().enumerate() {
            let product_id = registered_ids.get(i).unwrap();
            let stored_product = crate::storage::get_product(&env.env, product_id).unwrap();
            
            assert_eq!(stored_product.name.to_string(), *name);
            assert_eq!(stored_product.historical_demand.len(), demands.len());
            
            for (j, &expected_demand) in demands.iter().enumerate() {
                assert_eq!(stored_product.historical_demand.get(j).unwrap(), expected_demand);
            }
        }
    }
}

/// Test module for performance and scalability of product registration
mod product_registration_performance {
    use super::*;

    #[test]
    fn test_bulk_product_registration_performance() {
        let env = TestEnvironment::new();
        let perf_helper = PerformanceTestHelper::new(&env);
        
        let (registered_count, execution_time) = perf_helper.measure_execution_time(|| {
            let mut count = 0;
            
            for i in 0..100 {
                let product_name = String::from_str(&env.env, &format!("bulk_product_{}", i));
                let mut historical_demand = Vec::new(&env.env);
                
                // Add varying amounts of historical data
                let history_length = (i % 12) + 1; // 1-12 data points
                for j in 0..history_length {
                    historical_demand.push_back((i * 100 + j) as i128);
                }
                
                env.advance_time(1);
                
                let result = register_product(&env.env, product_name, historical_demand);
                if result.is_ok() {
                    count += 1;
                }
            }
            
            count
        });
        
        assert_eq!(registered_count, 100);
        assert!(execution_time < 10000, "Bulk registration took too long: {} ms", execution_time);
        
        // Verify all products are accessible
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert_eq!(all_product_ids.len(), 100);
    }

    #[test]
    fn test_large_historical_data_registration() {
        let env = TestEnvironment::new();
        let perf_helper = PerformanceTestHelper::new(&env);
        
        let product_name = String::from_str(&env.env, "large_history_product");
        let mut historical_demand = Vec::new(&env.env);
        
        // Create large historical dataset (5 years of daily data)
        let data_points = 5 * 365; // 1825 data points
        
        for i in 0..data_points {
            // Simulate seasonal variation
            let base_demand = 1000;
            let seasonal_factor = ((i as f64 * 2.0 * 3.14159 / 365.0).sin() * 200.0) as i128;
            let random_factor = (i % 100) as i128 - 50; // -50 to +49
            
            historical_demand.push_back(base_demand + seasonal_factor + random_factor);
        }
        
        let (product_id, execution_time) = perf_helper.measure_execution_time(|| {
            register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap()
        });
        
        assert!(execution_time < 5000, "Large data registration took too long: {} ms", execution_time);
        
        // Verify data integrity
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.historical_demand.len(), data_points as usize);
        assert_eq!(stored_product.name, product_name);
        
        // Spot check some values
        assert_eq!(stored_product.historical_demand.get(0).unwrap(), historical_demand.get(0).unwrap());
        assert_eq!(stored_product.historical_demand.get(500).unwrap(), historical_demand.get(500).unwrap());
        assert_eq!(stored_product.historical_demand.get(data_points - 1).unwrap(), historical_demand.get(data_points - 1).unwrap());
    }

    #[test]
    fn test_concurrent_product_registration_simulation() {
        let env = TestEnvironment::new();
        
        // Simulate concurrent registrations by registering products with minimal time gaps
        let concurrent_products = vec![
            "wheat_farm_a", "wheat_farm_b", "wheat_farm_c",
            "corn_region_1", "corn_region_2", "corn_region_3",
            "soy_cooperative_x", "soy_cooperative_y", "soy_cooperative_z",
        ];
        
        let mut registered_ids = Vec::new(&env.env);
        
        for product_name in concurrent_products.iter() {
            let name = String::from_str(&env.env, product_name);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(1000);
            historical_demand.push_back(1100);
            
            // Minimal time advancement to simulate near-concurrent registration
            env.advance_time(1);
            
            let product_id = register_product(&env.env, name, historical_demand).unwrap();
            registered_ids.push_back(product_id);
        }
        
        // Verify all registrations succeeded and are unique
        assert_eq!(registered_ids.len(), concurrent_products.len());
        
        for i in 0..registered_ids.len() {
            for j in i + 1..registered_ids.len() {
                assert_ne!(registered_ids.get(i).unwrap(), registered_ids.get(j).unwrap(),
                          "Products {} and {} have duplicate IDs", 
                          concurrent_products[i], concurrent_products[j]);
            }
        }
        
        // Verify all products are retrievable and have correct data
        for (i, product_id) in registered_ids.iter().enumerate() {
            let stored_product = crate::storage::get_product(&env.env, product_id).unwrap();
            assert_eq!(stored_product.name.to_string(), concurrent_products[i]);
            assert_eq!(stored_product.historical_demand.len(), 2);
        }
    }
}

/// Test module for edge cases and error scenarios
mod product_registration_edge_cases {
    use super::*;

    #[test]
    fn test_unicode_product_names() {
        let env = TestEnvironment::new();
        
        let unicode_names = vec![
            "小麦", // Chinese for wheat
            "القمح", // Arabic for wheat
            "пшеница", // Russian for wheat
            "🌾 grain", // Emoji in name
            "café", // Accented characters
            "piña", // Spanish with tilde
        ];
        
        for (i, name) in unicode_names.iter().enumerate() {
            let product_name = String::from_str(&env.env, name);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back((i as i128 + 1) * 100);
            
            env.advance_time(1);
            
            let result = register_product(&env.env, product_name.clone(), historical_demand);
            assert!(result.is_ok(), "Failed to register product with Unicode name: {}", name);
            
            let product_id = result.unwrap();
            let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
            assert_eq!(stored_product.name, product_name);
        }
    }

    #[test]
    fn test_very_long_product_name() {
        let env = TestEnvironment::new();
        
        // Create a very long product name
        let long_name = "a".repeat(1000);
        let product_name = String::from_str(&env.env, &long_name);
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(500);
        
        let result = register_product(&env.env, product_name.clone(), historical_demand);
        assert!(result.is_ok());
        
        let product_id = result.unwrap();
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.name.to_string().len(), 1000);
    }

    #[test]
    fn test_product_name_with_special_characters() {
        let env = TestEnvironment::new();
        
        let special_names = vec![
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
        
        for (i, name) in special_names.iter().enumerate() {
            let product_name = String::from_str(&env.env, name);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back((i as i128 + 1) * 50);
            
            env.advance_time(1);
            
            let result = register_product(&env.env, product_name.clone(), historical_demand);
            assert!(result.is_ok(), "Failed to register product with special characters: {}", name);
            
            let product_id = result.unwrap();
            let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
            assert_eq!(stored_product.name.to_string(), *name);
        }
    }

    #[test]
    fn test_product_registration_id_collision_resistance() {
        let env = TestEnvironment::new();
        
        // Test potential ID collision scenarios
        let similar_products = vec![
            ("product_a", 1000),
            ("product_b", 1000), // Same historical demand
            ("producta", 1001),   // Similar name
            ("product a", 1000),  // Space vs underscore
        ];
        
        let mut registered_ids = Vec::new(&env.env);
        let base_timestamp = env.current_timestamp();
        
        for (i, (name, demand)) in similar_products.iter().enumerate() {
            // Set specific timestamps to test collision resistance
            env.env.ledger().with_mut(|li| li.timestamp = base_timestamp + i as u64);
            
            let product_name = String::from_str(&env.env, name);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(*demand);
            
            let product_id = register_product(&env.env, product_name, historical_demand).unwrap();
            registered_ids.push_back(product_id);
        }
        
        // Verify all IDs are unique despite similar inputs
        for i in 0..registered_ids.len() {
            for j in i + 1..registered_ids.len() {
                assert_ne!(registered_ids.get(i).unwrap(), registered_ids.get(j).unwrap(),
                          "ID collision detected between products {} and {}", 
                          similar_products[i].0, similar_products[j].0);
            }
        }
    }
}

/// Integration tests combining product registration with other system components
mod integration_tests {
    use super::*;

    #[test]
    fn test_product_registration_with_subsequent_operations() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register a product
        let product_name = String::from_str(&env.env, "integration_wheat");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(800);
        historical_demand.push_back(900);
        historical_demand.push_back(1000);
        
        let product_id = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        // Simulate subsequent operations that depend on the product
        // (This would integrate with forecasting and recommendation modules)
        
        // Verify the product can be used for forecast generation
        let oracle = OracleSimulator::new(&env.env);
        let region = factory.mock_region("Integration-Region");
        
        // This would be the integration point with forecasting functionality
        // For now, we verify the product exists and has the correct structure
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.product_id, product_id);
        assert_eq!(stored_product.name, product_name);
        assert_eq!(stored_product.historical_demand.len(), 3);
        
        // Verify historical data can be used for trend analysis
        let mut trend_positive = true;
        for i in 1..stored_product.historical_demand.len() {
            let current = stored_product.historical_demand.get(i).unwrap();
            let previous = stored_product.historical_demand.get(i - 1).unwrap();
            if current <= previous {
                trend_positive = false;
                break;
            }
        }
        assert!(trend_positive, "Historical data should show positive trend");
    }

    #[test]
    fn test_product_registration_data_consistency_across_operations() {
        let env = TestEnvironment::new();
        
        // Register multiple related products
        let grain_products = vec![
            ("wheat_winter", vec![1000, 1100, 1200]),
            ("wheat_spring", vec![800, 900, 950]),
            ("barley_malting", vec![600, 650, 700]),
            ("barley_feed", vec![500, 550, 580]),
        ];
        
        let mut registered_products = Vec::new(&env.env);
        
        for (name, history) in grain_products.iter() {
            let product_name = String::from_str(&env.env, name);
            let mut historical_demand = Vec::new(&env.env);
            
            for &demand in history {
                historical_demand.push_back(demand);
            }
            
            env.advance_time(1);
            
            let product_id = register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
            registered_products.push_back((product_id, product_name, historical_demand));
        }
        
        // Verify data consistency across all registered products
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert_eq!(all_product_ids.len(), grain_products.len());
        
        for (product_id, expected_name, expected_history) in registered_products.iter() {
            assert!(all_product_ids.contains(product_id));
            
            let stored_product = crate::storage::get_product(&env.env, product_id).unwrap();
            assert_eq!(stored_product.name, *expected_name);
            assert_eq!(stored_product.historical_demand, *expected_history);
            assert_eq!(stored_product.product_id, *product_id);
        }
    }

    #[test]
    fn test_product_registration_with_complex_historical_patterns() {
        let env = TestEnvironment::new();
        
        // Test different historical demand patterns
        let pattern_tests = vec![
            ("steady_growth", generate_steady_growth_pattern(&env.env)),
            ("seasonal_cycle", generate_seasonal_pattern(&env.env)),
            ("volatile_market", generate_volatile_pattern(&env.env)),
            ("declining_demand", generate_declining_pattern(&env.env)),
            ("recovery_pattern", generate_recovery_pattern(&env.env)),
        ];
        
        for (pattern_name, historical_data) in pattern_tests.iter() {
            let product_name = String::from_str(&env.env, &format!("pattern_{}", pattern_name));
            
            env.advance_time(1);
            
            let product_id = register_product(&env.env, product_name.clone(), historical_data.clone()).unwrap();
            
            let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
            assert_eq!(stored_product.name, product_name);
            assert_eq!(stored_product.historical_demand, *historical_data);
            
            // Verify pattern characteristics are preserved
            validate_pattern_characteristics(pattern_name, &stored_product.historical_demand);
        }
    }
}

// Helper functions for pattern generation and validation
fn generate_steady_growth_pattern(env: &soroban_sdk::Env) -> soroban_sdk::Vec<i128> {
    let mut pattern = soroban_sdk::Vec::new(env);
    let base = 1000i128;
    let growth_rate = 50i128;
    
    for i in 0..12 {
        pattern.push_back(base + (i * growth_rate));
    }
    pattern
}

fn generate_seasonal_pattern(env: &soroban_sdk::Env) -> soroban_sdk::Vec<i128> {
    let mut pattern = soroban_sdk::Vec::new(env);
    let base = 1000i128;
    
    // Simulate 12 months with seasonal variation
    let seasonal_multipliers = vec![
        0.8, 0.9, 1.1, 1.3, 1.5, 1.2,  // Winter to Summer
        1.4, 1.3, 1.1, 0.9, 0.8, 0.7   // Summer to Winter
    ];
    
    for multiplier in seasonal_multipliers {
        pattern.push_back((base as f64 * multiplier) as i128);
    }
    pattern
}

fn generate_volatile_pattern(env: &soroban_sdk::Env) -> soroban_sdk::Vec<i128> {
    let mut pattern = soroban_sdk::Vec::new(env);
    let base = 1000i128;
    
    // Simulate high volatility with random-like changes
    let variations = vec![200, -150, 300, -250, 180, -100, 350, -200, 150, -80, 250, -180];
    
    for variation in variations {
        pattern.push_back(base + variation);
    }
    pattern
}

fn generate_declining_pattern(env: &soroban_sdk::Env) -> soroban_sdk::Vec<i128> {
    let mut pattern = soroban_sdk::Vec::new(env);
    let base = 1500i128;
    let decline_rate = 80i128;
    
    for i in 0..10 {
        pattern.push_back(base - (i * decline_rate));
    }
    pattern
}

fn generate_recovery_pattern(env: &soroban_sdk::Env) -> soroban_sdk::Vec<i128> {
    let mut pattern = soroban_sdk::Vec::new(env);
    
    // Start high, decline, then recover
    let recovery_data = vec![
        1200, 1000, 800, 600, 500, 400,  // Decline phase
        450, 550, 700, 900, 1100, 1300   // Recovery phase
    ];
    
    for value in recovery_data {
        pattern.push_back(value);
    }
    pattern
}

fn validate_pattern_characteristics(pattern_name: &str, data: &soroban_sdk::Vec<i128>) {
    match pattern_name {
        "steady_growth" => {
            // Verify consistent growth
            for i in 1..data.len() {
                let current = data.get(i).unwrap();
                let previous = data.get(i - 1).unwrap();
                assert!(current > previous, "Steady growth pattern violated at index {}", i);
            }
        },
        "seasonal_cycle" => {
            // Verify seasonal pattern has peaks and valleys
            assert!(data.len() >= 6, "Seasonal pattern too short");
            let max_val = data.iter().max().unwrap();
            let min_val = data.iter().min().unwrap();
            assert!(max_val - min_val > 300, "Seasonal variation too small");
        },
        "volatile_market" => {
            // Verify high volatility (frequent direction changes)
            let mut direction_changes = 0;
            for i in 2..data.len() {
                let current = data.get(i).unwrap();
                let previous = data.get(i - 1).unwrap();
                let before_previous = data.get(i - 2).unwrap();
                
                let trend1 = current > previous;
                let trend2 = previous > before_previous;
                
                if trend1 != trend2 {
                    direction_changes += 1;
                }
            }
            assert!(direction_changes >= 3, "Not enough volatility in volatile pattern");
        },
        "declining_demand" => {
            // Verify overall decline
            let first = data.get(0).unwrap();
            let last = data.get(data.len() - 1).unwrap();
            assert!(last < first, "Declining pattern should end lower than it started");
        },
        "recovery_pattern" => {
            // Verify decline followed by recovery
            assert!(data.len() >= 10, "Recovery pattern too short");
            let start = data.get(0).unwrap();
            let mid = data.get(data.len() / 2).unwrap();
            let end = data.get(data.len() - 1).unwrap();
            
            assert!(mid < start, "Recovery pattern should have declining phase");
            assert!(end > mid, "Recovery pattern should have recovery phase");
        },
        _ => {
            // Default validation - just ensure data exists
            assert!(!data.is_empty(), "Pattern data should not be empty");
        }
    }
}

/// Test module for stress testing and boundary conditions
mod stress_tests {
    use super::*;

    #[test]
    fn test_maximum_historical_data_points() {
        let env = TestEnvironment::new();
        
        let product_name = String::from_str(&env.env, "max_history_test");
        let mut historical_demand = Vec::new(&env.env);
        
        // Test with maximum reasonable historical data (10 years of daily data)
        let max_data_points = 10 * 365; // 3650 data points
        
        for i in 0..max_data_points {
            historical_demand.push_back(1000 + (i % 1000) as i128);
        }
        
        let result = register_product(&env.env, product_name.clone(), historical_demand.clone());
        assert!(result.is_ok(), "Should handle maximum historical data points");
        
        let product_id = result.unwrap();
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        
        assert_eq!(stored_product.historical_demand.len(), max_data_points as usize);
        assert_eq!(stored_product.name, product_name);
    }

    #[test]
    fn test_rapid_sequential_registrations() {
        let env = TestEnvironment::new();
        
        let mut product_ids = Vec::new(&env.env);
        let registration_count = 1000;
        
        for i in 0..registration_count {
            let product_name = String::from_str(&env.env, &format!("rapid_product_{}", i));
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(i as i128);
            
            // Minimal time advancement to simulate rapid registrations
            if i % 10 == 0 {
                env.advance_time(1);
            }
            
            let result = register_product(&env.env, product_name, historical_demand);
            assert!(result.is_ok(), "Rapid registration {} failed", i);
            
            product_ids.push_back(result.unwrap());
        }
        
        // Verify all registrations were successful and unique
        assert_eq!(product_ids.len(), registration_count as usize);
        
        // Check uniqueness
        for i in 0..product_ids.len() {
            for j in i + 1..product_ids.len() {
                assert_ne!(product_ids.get(i).unwrap(), product_ids.get(j).unwrap(),
                          "Duplicate IDs found at positions {} and {}", i, j);
            }
        }
        
        // Verify global product list integrity
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert_eq!(all_product_ids.len(), registration_count as usize);
    }

    #[test]
    fn test_memory_efficiency_with_large_datasets() {
        let env = TestEnvironment::new();
        
        // Register products with varying sizes of historical data
        let size_variants = vec![
            (1, "tiny"),
            (10, "small"),
            (100, "medium"),
            (1000, "large"),
            (5000, "extra_large"),
        ];
        
        for (size, size_name) in size_variants.iter() {
            let product_name = String::from_str(&env.env, &format!("size_test_{}", size_name));
            let mut historical_demand = Vec::new(&env.env);
            
            for i in 0..*size {
                historical_demand.push_back(1000 + i as i128);
            }
            
            env.advance_time(1);
            
            let result = register_product(&env.env, product_name.clone(), historical_demand.clone());
            assert!(result.is_ok(), "Failed to register {} size product", size_name);
            
            let product_id = result.unwrap();
            let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
            
            assert_eq!(stored_product.historical_demand.len(), *size as usize);
            assert_eq!(stored_product.name, product_name);
            
            // Verify data integrity for large datasets
            if *size >= 100 {
                // Check first, middle, and last values
                assert_eq!(stored_product.historical_demand.get(0).unwrap(), 1000);
                assert_eq!(stored_product.historical_demand.get(size / 2).unwrap(), 1000 + (size / 2) as i128);
                assert_eq!(stored_product.historical_demand.get(size - 1).unwrap(), 1000 + (size - 1) as i128);
            }
        }
    }
}

/// Test module for error recovery and system resilience
mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_product_registration_after_storage_operations() {
        let env = TestEnvironment::new();
        
        // Register initial product
        let product_name1 = String::from_str(&env.env, "initial_product");
        let mut historical_demand1 = Vec::new(&env.env);
        historical_demand1.push_back(1000);
        
        let product_id1 = register_product(&env.env, product_name1.clone(), historical_demand1.clone()).unwrap();
        
        // Verify it exists
        let stored_product1 = crate::storage::get_product(&env.env, &product_id1).unwrap();
        assert_eq!(stored_product1.name, product_name1);
        
        // Register additional products after storage operations
        for i in 2..=5 {
            let product_name = String::from_str(&env.env, &format!("product_{}", i));
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(1000 * i as i128);
            
            env.advance_time(1);
            
            let result = register_product(&env.env, product_name.clone(), historical_demand.clone());
            assert!(result.is_ok(), "Product registration {} failed after storage operations", i);
            
            let product_id = result.unwrap();
            let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
            assert_eq!(stored_product.name, product_name);
        }
        
        // Verify all products are accessible
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert_eq!(all_product_ids.len(), 5);
    }

    #[test]
    fn test_product_registration_consistency_after_failures() {
        let env = TestEnvironment::new();
        
        // Register some successful products
        let mut successful_products = Vec::new(&env.env);
        
        for i in 0..3 {
            let product_name = String::from_str(&env.env, &format!("success_product_{}", i));
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back((i + 1) as i128 * 100);
            
            env.advance_time(1);
            
            let product_id = register_product(&env.env, product_name, historical_demand).unwrap();
            successful_products.push_back(product_id);
        }
        
        // Attempt some registrations that should fail
        let empty_name = String::from_str(&env.env, "");
        let mut valid_history = Vec::new(&env.env);
        valid_history.push_back(500);
        
        let failed_result = register_product(&env.env, empty_name, valid_history);
        assert!(failed_result.is_err());
        
        // Register more successful products after failure
        for i in 3..6 {
            let product_name = String::from_str(&env.env, &format!("post_failure_product_{}", i));
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back((i + 1) as i128 * 100);
            
            env.advance_time(1);
            
            let product_id = register_product(&env.env, product_name, historical_demand).unwrap();
            successful_products.push_back(product_id);
        }
        
        // Verify system consistency
        assert_eq!(successful_products.len(), 6);
        let all_product_ids = crate::storage::get_all_product_ids(&env.env);
        assert_eq!(all_product_ids.len(), 6);
        
        // Verify all successful products are accessible
        for product_id in successful_products.iter() {
            let stored_product = crate::storage::get_product(&env.env, product_id);
            assert!(stored_product.is_ok(), "Product should be accessible after system operations");
        }
    }
}