#![cfg(test)]

use crate::{
    error::ContractError,
    storage::DemandForecast,
    forecasting::{generate_forecast, list_forecasts},
    tests::utils::*,
};
use soroban_sdk::{BytesN, String, Vec};

/// Test module for basic forecast generation functionality
mod forecast_generation {
    use super::*;

    #[test]
    fn test_successful_forecast_generation() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup - register a product first
        let product_name = String::from_str(&env.env, "wheat");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(1000);
        historical_demand.push_back(1200);
        historical_demand.push_back(1100);
        
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("US-West");
        let predicted_demand = 1500i128;
        let data_hash = factory.mock_data_hash("oracle_data_1");
        
        let result = generate_forecast(
            &env.env,
            product_id.clone(),
            region.clone(),
            predicted_demand,
            data_hash.clone(),
        );
        
        assert!(result.is_ok());
        let forecast_id = result.unwrap();
        
        // Verify forecast was created and stored correctly
        let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        assert_eq!(stored_forecast.forecast_id, forecast_id);
        assert_eq!(stored_forecast.product_id, product_id);
        assert_eq!(stored_forecast.region, region);
        assert_eq!(stored_forecast.predicted_demand, predicted_demand);
        assert_eq!(stored_forecast.data_hash, data_hash);
        assert!(stored_forecast.timestamp > 0);
    }

    #[test]
    fn test_forecast_generation_with_nonexistent_product() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        let nonexistent_product_id = factory.mock_product_id("nonexistent_product");
        let region = factory.mock_region("US-East");
        let predicted_demand = 1200i128;
        let data_hash = factory.mock_data_hash("test_data");
        
        let result = generate_forecast(
            &env.env,
            nonexistent_product_id,
            region,
            predicted_demand,
            data_hash,
        );
        
        TestAssertions::assert_contract_error(result, ContractError::ProductNotFound);
    }

    #[test]
    fn test_forecast_generation_with_negative_demand() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "corn");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(800);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("EU-North");
        let negative_demand = -100i128;
        let data_hash = factory.mock_data_hash("test_data");
        
        let result = generate_forecast(
            &env.env,
            product_id,
            region,
            negative_demand,
            data_hash,
        );
        
        TestAssertions::assert_contract_error(result, ContractError::InvalidData);
    }

    #[test]
    fn test_forecast_generation_with_zero_demand() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "rice");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(600);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("Asia-Pacific");
        let zero_demand = 0i128;
        let data_hash = factory.mock_data_hash("test_data");
        
        let result = generate_forecast(
            &env.env,
            product_id,
            region,
            zero_demand,
            data_hash,
        );
        
        TestAssertions::assert_contract_error(result, ContractError::InvalidData);
    }

    #[test]
    fn test_forecast_generation_with_minimum_valid_demand() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "soybeans");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(500);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("Latin-America");
        let min_valid_demand = 1i128; // Minimum valid demand
        let data_hash = factory.mock_data_hash("test_data");
        
        let result = generate_forecast(
            &env.env,
            product_id.clone(),
            region.clone(),
            min_valid_demand,
            data_hash.clone(),
        );
        
        assert!(result.is_ok());
        let forecast_id = result.unwrap();
        
        let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        assert_eq!(stored_forecast.predicted_demand, min_valid_demand);
    }

    #[test]
    fn test_forecast_generation_with_maximum_demand() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "barley");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(750);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("EU-South");
        let max_demand = i128::MAX; // Maximum possible demand
        let data_hash = factory.mock_data_hash("test_data");
        
        let result = generate_forecast(
            &env.env,
            product_id.clone(),
            region.clone(),
            max_demand,
            data_hash.clone(),
        );
        
        assert!(result.is_ok());
        let forecast_id = result.unwrap();
        
        let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        assert_eq!(stored_forecast.predicted_demand, max_demand);
    }

    #[test]
    fn test_forecast_id_uniqueness() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "oats");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(400);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("Test-Region");
        let predicted_demand = 900i128;
        let data_hash = factory.mock_data_hash("test_data");
        
        // Generate first forecast
        let forecast_id1 = generate_forecast(
            &env.env,
            product_id.clone(),
            region.clone(),
            predicted_demand,
            data_hash.clone(),
        ).unwrap();
        
        // Advance time to ensure different timestamp
        env.advance_time(1);
        
        // Generate second forecast with same parameters but different timestamp
        let forecast_id2 = generate_forecast(
            &env.env,
            product_id,
            region,
            predicted_demand,
            data_hash,
        ).unwrap();
        
        // IDs should be different due to different timestamps
        assert_ne!(forecast_id1, forecast_id2);
        
        // Both forecasts should exist independently
        let forecast1 = crate::storage::get_forecast(&env.env, &forecast_id1).unwrap();
        let forecast2 = crate::storage::get_forecast(&env.env, &forecast_id2).unwrap();
        
        assert_eq!(forecast1.predicted_demand, predicted_demand);
        assert_eq!(forecast2.predicted_demand, predicted_demand);
        assert!(forecast2.timestamp > forecast1.timestamp);
    }

    #[test]
    fn test_forecast_generation_different_regions() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "cotton");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(300);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let regions = ["US-South", "India", "Egypt", "Brazil"];
        let mut forecast_ids = Vec::new(&env.env);
        
        for (i, region_name) in regions.iter().enumerate() {
            let region = factory.mock_region(region_name);
            let predicted_demand = (1000 + i * 100) as i128;
            let data_hash = factory.mock_data_hash(&format!("data_{}", i));
            
            env.advance_time(1); // Ensure unique timestamps
            
            let forecast_id = generate_forecast(
                &env.env,
                product_id.clone(),
                region.clone(),
                predicted_demand,
                data_hash,
            ).unwrap();
            
            forecast_ids.push_back(forecast_id.clone());
            
            // Verify forecast is stored correctly
            let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
            assert_eq!(stored_forecast.region, region);
            assert_eq!(stored_forecast.predicted_demand, predicted_demand);
        }
        
        // Verify all forecast IDs are unique
        for i in 0..forecast_ids.len() {
            for j in i + 1..forecast_ids.len() {
                assert_ne!(forecast_ids.get(i).unwrap(), forecast_ids.get(j).unwrap());
            }
        }
    }
}

/// Test module for forecast data validation and integrity
mod forecast_data_validation {
    use super::*;

    #[test]
    fn test_forecast_timestamp_accuracy() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "timestamp_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(500);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("Timestamp-Region");
        let predicted_demand = 1200i128;
        let data_hash = factory.mock_data_hash("timestamp_data");
        
        let before_timestamp = env.current_timestamp();
        
        let forecast_id = generate_forecast(
            &env.env,
            product_id,
            region,
            predicted_demand,
            data_hash,
        ).unwrap();
        
        let after_timestamp = env.current_timestamp();
        let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        
        // Timestamp should be within the execution window
        assert!(stored_forecast.timestamp >= before_timestamp);
        assert!(stored_forecast.timestamp <= after_timestamp);
    }

    #[test]
    fn test_forecast_data_hash_preservation() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "hash_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(700);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("Hash-Region");
        let predicted_demand = 1300i128;
        
        // Create specific data hash
        let test_data = "specific_oracle_data_12345";
        let expected_hash = env.env.crypto().sha256(&String::from_str(&env.env, test_data));
        
        let forecast_id = generate_forecast(
            &env.env,
            product_id,
            region,
            predicted_demand,
            expected_hash.clone(),
        ).unwrap();
        
        let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        assert_eq!(stored_forecast.data_hash, expected_hash);
    }

    #[test]
    fn test_forecast_with_various_demand_ranges() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "range_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(1000);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let demand_ranges = vec![
            (1, "minimal"),
            (100, "low"),
            (1_000, "moderate"),
            (10_000, "high"),
            (100_000, "very_high"),
            (1_000_000, "extreme"),
            (1_000_000_000, "massive"),
        ];
        
        for (demand, range_name) in demand_ranges.iter() {
            let region = factory.mock_region(&format!("Range-{}", range_name));
            let data_hash = factory.mock_data_hash(&format!("data_{}", range_name));
            
            env.advance_time(1);
            
            let result = generate_forecast(
                &env.env,
                product_id.clone(),
                region.clone(),
                *demand,
                data_hash,
            );
            
            assert!(result.is_ok(), "Failed to generate forecast for {} demand", range_name);
            
            let forecast_id = result.unwrap();
            let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
            assert_eq!(stored_forecast.predicted_demand, *demand);
        }
    }

    #[test]
    fn test_forecast_storage_indexing() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "indexing_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(800);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("Indexing-Region");
        let predicted_demand = 1400i128;
        let data_hash = factory.mock_data_hash("indexing_data");
        
        let forecast_id = generate_forecast(
            &env.env,
            product_id,
            region.clone(),
            predicted_demand,
            data_hash,
        ).unwrap();
        
        // Verify forecast is indexed globally
        let all_forecast_ids = crate::storage::get_all_forecast_ids(&env.env);
        assert!(all_forecast_ids.contains(&forecast_id));
        
        // Verify forecast is indexed by region
        let region_forecast_ids = crate::storage::get_region_forecast_ids(&env.env, &region);
        assert!(region_forecast_ids.contains(&forecast_id));
    }
}

/// Test module for forecast listing and filtering functionality
mod forecast_listing {
    use super::*;

    #[test]
    fn test_list_all_forecasts() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register multiple products
        let products = vec![
            ("wheat", 1000),
            ("corn", 800),
            ("soybeans", 600),
        ];
        
        let mut expected_forecasts = Vec::new(&env.env);
        
        for (product_name, base_demand) in products.iter() {
            let name = String::from_str(&env.env, product_name);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(*base_demand);
            let product_id = crate::data::register_product(&env.env, name, historical_demand).unwrap();
            
            let region = factory.mock_region(&format!("{}-region", product_name));
            let predicted_demand = (*base_demand + 200) as i128;
            let data_hash = factory.mock_data_hash(&format!("{}_data", product_name));
            
            env.advance_time(1);
            
            let forecast_id = generate_forecast(
                &env.env,
                product_id,
                region,
                predicted_demand,
                data_hash,
            ).unwrap();
            
            expected_forecasts.push_back(forecast_id);
        }
        
        // List all forecasts without filters
        let all_forecasts = list_forecasts(&env.env, None, None);
        
        assert_eq!(all_forecasts.len(), products.len());
        
        // Verify all expected forecasts are present
        for expected_id in expected_forecasts.iter() {
            let found = all_forecasts.iter().any(|f| f.forecast_id == *expected_id);
            assert!(found, "Expected forecast not found in list");
        }
    }

    #[test]
    fn test_list_forecasts_filtered_by_product() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register two products
        let wheat_name = String::from_str(&env.env, "wheat");
        let mut wheat_history = Vec::new(&env.env);
        wheat_history.push_back(1000);
        let wheat_id = crate::data::register_product(&env.env, wheat_name, wheat_history).unwrap();
        
        let corn_name = String::from_str(&env.env, "corn");
        let mut corn_history = Vec::new(&env.env);
        corn_history.push_back(800);
        let corn_id = crate::data::register_product(&env.env, corn_name, corn_history).unwrap();
        
        // Generate forecasts for both products
        let wheat_region = factory.mock_region("Wheat-Region");
        let wheat_forecast_id = generate_forecast(
            &env.env,
            wheat_id.clone(),
            wheat_region,
            1200,
            factory.mock_data_hash("wheat_data"),
        ).unwrap();
        
        env.advance_time(1);
        
        let corn_region = factory.mock_region("Corn-Region");
        let _corn_forecast_id = generate_forecast(
            &env.env,
            corn_id,
            corn_region,
            1000,
            factory.mock_data_hash("corn_data"),
        ).unwrap();
        
        // Filter by wheat product
        let wheat_forecasts = list_forecasts(&env.env, Some(wheat_id), None);
        
        assert_eq!(wheat_forecasts.len(), 1);
        assert_eq!(wheat_forecasts.get(0).unwrap().forecast_id, wheat_forecast_id);
        assert_eq!(wheat_forecasts.get(0).unwrap().product_id, wheat_id);
    }

    #[test]
    fn test_list_forecasts_filtered_by_region() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "regional_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(900);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        // Generate forecasts for different regions
        let us_region = factory.mock_region("US-Region");
        let eu_region = factory.mock_region("EU-Region");
        
        let us_forecast_id = generate_forecast(
            &env.env,
            product_id.clone(),
            us_region.clone(),
            1300,
            factory.mock_data_hash("us_data"),
        ).unwrap();
        
        env.advance_time(1);
        
        let _eu_forecast_id = generate_forecast(
            &env.env,
            product_id,
            eu_region,
            1100,
            factory.mock_data_hash("eu_data"),
        ).unwrap();
        
        // Filter by US region
        let us_forecasts = list_forecasts(&env.env, None, Some(us_region.clone()));
        
        assert_eq!(us_forecasts.len(), 1);
        assert_eq!(us_forecasts.get(0).unwrap().forecast_id, us_forecast_id);
        assert_eq!(us_forecasts.get(0).unwrap().region, us_region);
    }

    #[test]
    fn test_list_forecasts_filtered_by_product_and_region() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register two products
        let wheat_name = String::from_str(&env.env, "wheat_filter");
        let mut wheat_history = Vec::new(&env.env);
        wheat_history.push_back(1000);
        let wheat_id = crate::data::register_product(&env.env, wheat_name, wheat_history).unwrap();
        
        let corn_name = String::from_str(&env.env, "corn_filter");
        let mut corn_history = Vec::new(&env.env);
        corn_history.push_back(800);
        let corn_id = crate::data::register_product(&env.env, corn_name, corn_history).unwrap();
        
        let us_region = factory.mock_region("US-Filter");
        let eu_region = factory.mock_region("EU-Filter");
        
        // Generate forecasts for all combinations
        let wheat_us_id = generate_forecast(
            &env.env,
            wheat_id.clone(),
            us_region.clone(),
            1400,
            factory.mock_data_hash("wheat_us"),
        ).unwrap();
        
        env.advance_time(1);
        
        let _wheat_eu_id = generate_forecast(
            &env.env,
            wheat_id.clone(),
            eu_region.clone(),
            1300,
            factory.mock_data_hash("wheat_eu"),
        ).unwrap();
        
        env.advance_time(1);
        
        let _corn_us_id = generate_forecast(
            &env.env,
            corn_id.clone(),
            us_region.clone(),
            1100,
            factory.mock_data_hash("corn_us"),
        ).unwrap();
        
        env.advance_time(1);
        
        let _corn_eu_id = generate_forecast(
            &env.env,
            corn_id,
            eu_region,
            1000,
            factory.mock_data_hash("corn_eu"),
        ).unwrap();
        
        // Filter by wheat product and US region
        let filtered_forecasts = list_forecasts(&env.env, Some(wheat_id), Some(us_region.clone()));
        
        assert_eq!(filtered_forecasts.len(), 1);
        assert_eq!(filtered_forecasts.get(0).unwrap().forecast_id, wheat_us_id);
        assert_eq!(filtered_forecasts.get(0).unwrap().product_id, wheat_id);
        assert_eq!(filtered_forecasts.get(0).unwrap().region, us_region);
    }

    #[test]
    fn test_list_forecasts_empty_results() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product but don't generate any forecasts
        let product_name = String::from_str(&env.env, "empty_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(500);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        // List forecasts should return empty
        let all_forecasts = list_forecasts(&env.env, None, None);
        assert_eq!(all_forecasts.len(), 0);
        
        // Filter by product should return empty
        let product_forecasts = list_forecasts(&env.env, Some(product_id), None);
        assert_eq!(product_forecasts.len(), 0);
        
        // Filter by region should return empty
        let region = factory.mock_region("NonExistent-Region");
        let region_forecasts = list_forecasts(&env.env, None, Some(region));
        assert_eq!(region_forecasts.len(), 0);
    }

    #[test]
    fn test_list_forecasts_nonexistent_filters() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product and generate forecast
        let product_name = String::from_str(&env.env, "existing_product");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(700);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let region = factory.mock_region("Existing-Region");
        let _forecast_id = generate_forecast(
            &env.env,
            product_id,
            region,
            1200,
            factory.mock_data_hash("existing_data"),
        ).unwrap();
        
        // Filter by nonexistent product
        let nonexistent_product_id = factory.mock_product_id("nonexistent");
        let no_forecasts = list_forecasts(&env.env, Some(nonexistent_product_id), None);
        assert_eq!(no_forecasts.len(), 0);
        
        // Filter by nonexistent region
        let nonexistent_region = factory.mock_region("Nonexistent-Region");
        let no_forecasts = list_forecasts(&env.env, None, Some(nonexistent_region));
        assert_eq!(no_forecasts.len(), 0);
    }
}

/// Test module for performance and scalability of forecasting operations
mod forecast_performance {
    use super::*;

    #[test]
    fn test_bulk_forecast_generation_performance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let perf_helper = PerformanceTestHelper::new(&env);
        
        // Register multiple products
        let mut product_ids = Vec::new(&env.env);
        for i in 0..10 {
            let product_name = String::from_str(&env.env, &format!("perf_product_{}", i));
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back((i + 1) as i128 * 100);
            let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
            product_ids.push_back(product_id);
        }
        
        let (forecast_count, execution_time) = perf_helper.measure_execution_time(|| {
            let mut count = 0;
            
            for i in 0..100 {
                let product_idx = i % product_ids.len();
                let product_id = product_ids.get(product_idx).unwrap();
                
                let region = factory.mock_region(&format!("perf_region_{}", i % 5));
                let predicted_demand = (1000 + i * 10) as i128;
                let data_hash = factory.mock_data_hash(&format!("perf_data_{}", i));
                
                env.advance_time(1);
                
                let result = generate_forecast(
                    &env.env,
                    product_id.clone(),
                    region,
                    predicted_demand,
                    data_hash,
                );
                
                if result.is_ok() {
                    count += 1;
                }
            }
            
            count
        });
        
        assert_eq!(forecast_count, 100);
        assert!(execution_time < 10000, "Bulk forecast generation took too long: {} ms", execution_time);
        
        // Verify all forecasts were created
        let all_forecasts = list_forecasts(&env.env, None, None);
        assert_eq!(all_forecasts.len(), 100);
    }

    #[test]
    fn test_forecast_listing_performance() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        let perf_helper = PerformanceTestHelper::new(&env);
        
        // Setup - create many forecasts
        let product_name = String::from_str(&env.env, "listing_perf_product");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(1000);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        // Generate 200 forecasts across different regions
        for i in 0..200 {
            let region = factory.mock_region(&format!("listing_region_{}", i % 10));
            let predicted_demand = (1000 + i * 5) as i128;
            let data_hash = factory.mock_data_hash(&format!("listing_data_{}", i));
            
            env.advance_time(1);
            
            let _ = generate_forecast(
                &env.env,
                product_id.clone(),
                region,
                predicted_demand,
                data_hash,
            ).unwrap();
        }
        
        // Test listing performance
        let (all_forecasts, list_all_time) = perf_helper.measure_execution_time(|| {
            list_forecasts(&env.env, None, None)
        });
        
        assert_eq!(all_forecasts.len(), 200);
        assert!(list_all_time < 3000, "Listing all forecasts took too long: {} ms", list_all_time);
        
        // Test filtered listing performance
        let (filtered_forecasts, filter_time) = perf_helper.measure_execution_time(|| {
            list_forecasts(&env.env, Some(product_id.clone()), None)
        });
        
        assert_eq!(filtered_forecasts.len(), 200);
        assert!(filter_time < 3000, "Filtered listing took too long: {} ms", filter_time);
        
        // Test region filtering performance
        let test_region = factory.mock_region("listing_region_5");
        let (region_forecasts, region_time) = perf_helper.measure_execution_time(|| {
            list_forecasts(&env.env, None, Some(test_region))
        });
        
        assert_eq!(region_forecasts.len(), 20); // 200 forecasts / 10 regions = 20 per region
        assert!(region_time < 2000, "Region filtering took too long: {} ms", region_time);
    }

    #[test]
    fn test_forecast_generation_memory_efficiency() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "memory_test_product");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(800);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        // Generate many forecasts with large data hashes
        let mut forecast_ids = Vec::new(&env.env);
        
        for i in 0..500 {
            let region = factory.mock_region(&format!("memory_region_{}", i % 20));
            let predicted_demand = (1000 + i * 2) as i128;
            
            // Create large data hash to test memory efficiency
            let large_data = format!("large_oracle_data_with_lots_of_information_{}_end", i);
            let data_hash = env.env.crypto().sha256(&String::from_str(&env.env, &large_data));
            
            env.advance_time(1);
            
            let forecast_id = generate_forecast(
                &env.env,
                product_id.clone(),
                region,
                predicted_demand,
                data_hash,
            ).unwrap();
            
            forecast_ids.push_back(forecast_id);
        }
        
        // Verify all forecasts are accessible
        assert_eq!(forecast_ids.len(), 500);
        
        // Spot check some forecasts to ensure data integrity
        for i in [0, 100, 250, 499].iter() {
            let forecast_id = forecast_ids.get(*i).unwrap();
            let forecast = crate::storage::get_forecast(&env.env, forecast_id);
            assert!(forecast.is_ok(), "Forecast {} should be accessible", i);
            
            let stored_forecast = forecast.unwrap();
            assert_eq!(stored_forecast.predicted_demand, (1000 + i * 2) as i128);
        }
        
        // Test listing performance with large dataset
        let all_forecasts = list_forecasts(&env.env, None, None);
        assert_eq!(all_forecasts.len(), 500);
    }
}

/// Test module for edge cases and error scenarios
mod forecast_edge_cases {
    use super::*;

    #[test]
    fn test_forecast_generation_with_special_characters_in_region() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "special_char_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(600);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let special_regions = vec![
            "Region@Special",
            "Region#Hash",
            "Region$Dollar",
            "Region%Percent",
            "Region&Ampersand",
            "Region*Star",
            "Region+Plus",
            "Region=Equals",
            "Region[Bracket]",
            "Region{Brace}",
            "Region|Pipe",
            "Region\\Backslash",
            "Region:Colon",
            "Region;Semicolon",
            "Region\"Quote",
            "Region'Apostrophe",
            "Region<Less>",
            "Region,Comma",
            "Region.Dot",
            "Region?Question",
            "Region/Slash",
            "Region~Tilde",
            "Region`Backtick",
        ];
        
        for (i, region_name) in special_regions.iter().enumerate() {
            let region = String::from_str(&env.env, region_name);
            let predicted_demand = (1000 + i * 50) as i128;
            let data_hash = factory.mock_data_hash(&format!("special_data_{}", i));
            
            env.advance_time(1);
            
            let result = generate_forecast(
                &env.env,
                product_id.clone(),
                region.clone(),
                predicted_demand,
                data_hash,
            );
            
            assert!(result.is_ok(), "Failed to generate forecast for region with special characters: {}", region_name);
            
            let forecast_id = result.unwrap();
            let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
            assert_eq!(stored_forecast.region, region);
        }
    }

    #[test]
    fn test_forecast_generation_with_unicode_regions() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "unicode_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(750);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let unicode_regions = vec![
            "中国北方", // Chinese
            "الشرق الأوسط", // Arabic
            "Европа", // Russian
            "🌍 Global", // Emoji
            "Amérique du Sud", // French with accents
            "España", // Spanish with tilde
            "日本", // Japanese
            "한국", // Korean
        ];
        
        for (i, region_name) in unicode_regions.iter().enumerate() {
            let region = String::from_str(&env.env, region_name);
            let predicted_demand = (1200 + i * 100) as i128;
            let data_hash = factory.mock_data_hash(&format!("unicode_data_{}", i));
            
            env.advance_time(1);
            
            let result = generate_forecast(
                &env.env,
                product_id.clone(),
                region.clone(),
                predicted_demand,
                data_hash,
            );
            
            assert!(result.is_ok(), "Failed to generate forecast for Unicode region: {}", region_name);
            
            let forecast_id = result.unwrap();
            let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
            assert_eq!(stored_forecast.region, region);
        }
    }

    #[test]
    fn test_forecast_generation_with_very_long_region_name() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "long_region_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(900);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        // Create very long region name
        let long_region_name = format!("VeryLongRegionNameThatExceedsNormalLengthLimits{}", "Repeated".repeat(100));
        let region = String::from_str(&env.env, &long_region_name);
        let predicted_demand = 1500i128;
        let data_hash = factory.mock_data_hash("long_region_data");
        
        let result = generate_forecast(
            &env.env,
            product_id,
            region.clone(),
            predicted_demand,
            data_hash,
        );
        
        assert!(result.is_ok(), "Failed to generate forecast for very long region name");
        
        let forecast_id = result.unwrap();
        let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        assert_eq!(stored_forecast.region, region);
    }

    #[test]
    fn test_forecast_generation_with_empty_region() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "empty_region_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(500);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        let empty_region = String::from_str(&env.env, "");
        let predicted_demand = 1100i128;
        let data_hash = factory.mock_data_hash("empty_region_data");
        
        let result = generate_forecast(
            &env.env,
            product_id,
            empty_region.clone(),
            predicted_demand,
            data_hash,
        );
        
        // Should succeed with empty region
        assert!(result.is_ok(), "Failed to generate forecast for empty region");
        
        let forecast_id = result.unwrap();
        let stored_forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        assert_eq!(stored_forecast.region, empty_region);
    }

    #[test]
    fn test_forecast_retrieval_edge_cases() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test retrieving nonexistent forecast
        let nonexistent_id = factory.mock_data_hash("nonexistent_forecast");
        let result = crate::storage::get_forecast(&env.env, &nonexistent_id);
        assert!(result.is_err());
        
        // Test listing with filters that match nothing
        let empty_results = list_forecasts(&env.env, None, Some(String::from_str(&env.env, "NonExistentRegion")));
        assert_eq!(empty_results.len(), 0);
        
        let nonexistent_product = factory.mock_product_id("nonexistent_product");
        let empty_results = list_forecasts(&env.env, Some(nonexistent_product), None);
        assert_eq!(empty_results.len(), 0);
    }
}

/// Test module for concurrent operations and race conditions
mod forecast_concurrency {
    use super::*;

    #[test]
    fn test_concurrent_forecast_generation_same_product() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "concurrent_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(1000);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        // Simulate concurrent forecast generation for same product, different regions
        let regions = ["Region-A", "Region-B", "Region-C", "Region-D"];
        let mut forecast_ids = Vec::new(&env.env);
        
        for (i, region_name) in regions.iter().enumerate() {
            let region = factory.mock_region(region_name);
            let predicted_demand = (1200 + i * 100) as i128;
            let data_hash = factory.mock_data_hash(&format!("concurrent_data_{}", i));
            
            // Minimal time advancement to simulate near-concurrent operations
            env.advance_time(1);
            
            let forecast_id = generate_forecast(
                &env.env,
                product_id.clone(),
                region,
                predicted_demand,
                data_hash,
            ).unwrap();
            
            forecast_ids.push_back(forecast_id);
        }
        
        // Verify all forecasts were created successfully and are unique
        assert_eq!(forecast_ids.len(), regions.len());
        
        for i in 0..forecast_ids.len() {
            for j in i + 1..forecast_ids.len() {
                assert_ne!(forecast_ids.get(i).unwrap(), forecast_ids.get(j).unwrap(),
                          "Concurrent forecasts for regions {} and {} have duplicate IDs", 
                          regions[i], regions[j]);
            }
        }
        
        // Verify all forecasts are retrievable
        for (i, forecast_id) in forecast_ids.iter().enumerate() {
            let forecast = crate::storage::get_forecast(&env.env, forecast_id).unwrap();
            assert_eq!(forecast.product_id, product_id);
            assert_eq!(forecast.predicted_demand, (1200 + i * 100) as i128);
        }
    }

    #[test]
    fn test_concurrent_forecast_generation_different_products() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register multiple products
        let products = vec!["wheat", "corn", "soybeans", "rice"];
        let mut product_ids = Vec::new(&env.env);
        
        for product_name in products.iter() {
            let name = String::from_str(&env.env, product_name);
            let mut historical_demand = Vec::new(&env.env);
            historical_demand.push_back(800);
            let product_id = crate::data::register_product(&env.env, name, historical_demand).unwrap();
            product_ids.push_back(product_id);
        }
        
        // Generate forecasts concurrently for different products
        let mut all_forecast_ids = Vec::new(&env.env);
        let region = factory.mock_region("Concurrent-Region");
        
        for (i, product_id) in product_ids.iter().enumerate() {
            let predicted_demand = (1000 + i * 200) as i128;
            let data_hash = factory.mock_data_hash(&format!("multi_product_data_{}", i));
            
            env.advance_time(1);
            
            let forecast_id = generate_forecast(
                &env.env,
                product_id.clone(),
                region.clone(),
                predicted_demand,
                data_hash,
            ).unwrap();
            
            all_forecast_ids.push_back(forecast_id);
        }
        
        // Verify all forecasts are unique and correct
        assert_eq!(all_forecast_ids.len(), products.len());
        
        for i in 0..all_forecast_ids.len() {
            let forecast = crate::storage::get_forecast(&env.env, &all_forecast_ids.get(i).unwrap()).unwrap();
            assert_eq!(forecast.product_id, *product_ids.get(i).unwrap());
            assert_eq!(forecast.predicted_demand, (1000 + i * 200) as i128);
        }
        
        // Test listing all forecasts
        let all_forecasts = list_forecasts(&env.env, None, None);
        assert_eq!(all_forecasts.len(), products.len());
    }

    #[test]
    fn test_forecast_operations_isolation() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register product
        let product_name = String::from_str(&env.env, "isolation_test");
        let mut historical_demand = Vec::new(&env.env);
        historical_demand.push_back(1000);
        let product_id = crate::data::register_product(&env.env, product_name, historical_demand).unwrap();
        
        // Generate initial forecast
        let region = factory.mock_region("Isolation-Region");
        let forecast_id1 = generate_forecast(
            &env.env,
            product_id.clone(),
            region.clone(),
            1300,
            factory.mock_data_hash("isolation_data_1"),
        ).unwrap();
        
        // Verify initial state
        let forecast1 = crate::storage::get_forecast(&env.env, &forecast_id1).unwrap();
        assert_eq!(forecast1.predicted_demand, 1300);
        
        env.advance_time(5);
        
        // Generate second forecast
        let forecast_id2 = generate_forecast(
            &env.env,
            product_id,
            region,
            1500,
            factory.mock_data_hash("isolation_data_2"),
        ).unwrap();
        
        // Verify both forecasts exist independently
        let forecast1_after = crate::storage::get_forecast(&env.env, &forecast_id1).unwrap();
        let forecast2 = crate::storage::get_forecast(&env.env, &forecast_id2).unwrap();
        
        assert_eq!(forecast1_after.predicted_demand, 1300); // Unchanged
        assert_eq!(forecast2.predicted_demand, 1500);
        assert_ne!(forecast_id1, forecast_id2);
        assert!(forecast2.timestamp > forecast1_after.timestamp);
        
        // Verify listing includes both
        let all_forecasts = list_forecasts(&env.env, None, None);
        assert_eq!(all_forecasts.len(), 2);
    }
}

/// Integration tests combining forecasting with other modules
mod forecasting_integration {
    use super::*;

    #[test]
    fn test_forecasting_with_product_registration_integration() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Test the complete flow: register product -> generate forecast
        let product_name = String::from_str(&env.env, "integration_wheat");
        let mut historical_demand = Vec::new(&env.env);
        for i in 0..12 {
            historical_demand.push_back(1000 + (i * 50)); // Growing trend
        }
        
        let product_id = crate::data::register_product(&env.env, product_name.clone(), historical_demand.clone()).unwrap();
        
        // Verify product exists and can be used for forecasting
        let stored_product = crate::storage::get_product(&env.env, &product_id).unwrap();
        assert_eq!(stored_product.name, product_name);
        
        // Generate forecast for the registered product
        let region = factory.mock_region("Integration-Region");
        let predicted_demand = 1800i128;
        let data_hash = factory.mock_data_hash("integration_data");
        
        let forecast_id = generate_forecast(
            &env.env,
            product_id.clone(),
            region.clone(),
            predicted_demand,
            data_hash.clone(),
        ).unwrap();
        
        // Verify forecast references correct product
        let forecast = crate::storage::get_forecast(&env.env, &forecast_id).unwrap();
        assert_eq!(forecast.product_id, product_id);
        assert_eq!(forecast.region, region);
        assert_eq!(forecast.predicted_demand, predicted_demand);
        assert_eq!(forecast.data_hash, data_hash);
        
        // Verify forecast appears in listings
        let all_forecasts = list_forecasts(&env.env, None, None);
        assert_eq!(all_forecasts.len(), 1);
        assert_eq!(all_forecasts.get(0).unwrap().forecast_id, forecast_id);
        
        let product_forecasts = list_forecasts(&env.env, Some(product_id), None);
        assert_eq!(product_forecasts.len(), 1);
        
        let region_forecasts = list_forecasts(&env.env, None, Some(region));
        assert_eq!(region_forecasts.len(), 1);
    }

    #[test]
    fn test_forecasting_data_consistency_across_operations() {
        let env = TestEnvironment::new();
        let factory = TestDataFactory::new(&env.env);
        
        // Register multiple products with different characteristics
        let products_data = vec![
            ("wheat_premium", vec![1000, 1100, 1200, 1300]),
            ("corn_hybrid", vec![800, 850, 900, 950]),
            ("soybeans_organic", vec![600, 650, 700, 750]),
        ];
        
        let mut product_forecast_map = Vec::new(&env.env);
        
        for (product_name, history) in products_data.iter() {
            // Register product
            let name = String::from_str(&env.env, product_name);
            let mut historical_demand = Vec::new(&env.env);
            for &demand in history {
                historical_demand.push_back(demand);
            }
            let product_id = crate::data::register_product(&env.env, name, historical_demand).unwrap();
            
            // Generate forecasts for multiple regions
            let mut product_forecasts = Vec::new(&env.env);
            let regions = ["North", "South", "East", "West"];
            
            for (i, region_name) in regions.iter().enumerate() {
                let region = factory.mock_region(&format!("{}-{}", product_name, region_name));
                let base_demand = history.last().unwrap_or(&1000);
                let predicted_demand = base_demand + ((i + 1) * 100) as i128;
                let data_hash = factory.mock_data_hash(&format!("{}_{}_data", product_name, region_name));
                
                env.advance_time(1);
                
                let forecast_id = generate_forecast(
                    &env.env,
                    product_id.clone(),
                    region,
                    predicted_demand,
                    data_hash,
                ).unwrap();
                
                product_forecasts.push_back(forecast_id);
            }
            
            product_forecast_map.push_back((product_id, product_forecasts));
        }
        
        // Verify data consistency across all operations
        for (product_id, forecast_ids) in product_forecast_map.iter() {
            // Verify product still exists
            let product = crate::storage::get_product(&env.env, product_id);
            assert!(product.is_ok(), "Product should still exist");
            
            // Verify all forecasts for this product
            for forecast_id in forecast_ids.iter() {
                let forecast = crate::storage::get_forecast(&env.env, forecast_id).unwrap();
                assert_eq!(forecast.product_id, *product_id);
                assert!(forecast.predicted_demand > 0);
            }
            
            // Verify product filtering works
            let product_forecasts = list_forecasts(&env.env, Some(product_id.clone()), None);
            assert_eq!(product_forecasts.len(), 4); // 4 regions per product
        }
        
        // Verify global consistency
        let all_forecasts = list_forecasts(&env.env, None, None);
        assert_eq!(all_forecasts.len(), 12); // 3 products × 4 regions
        
        // Verify all forecasts have valid product references
        for forecast in all_forecasts.iter() {
            let product = crate::storage::get_product(&env.env, &forecast.product_id);
            assert!(product.is_ok(), "Forecast should reference valid product");
        }
    }
}