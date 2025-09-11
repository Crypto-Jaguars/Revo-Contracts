//! Comprehensive tests for crop recommendation generation and accessibility

#![cfg(test)]

use crate::{
    error::ContractError,
    storage::{Recommendation, RecommendationType, Priority},
    tests::utils::*,
};
use soroban_sdk::{BytesN, String, Vec, Address};

/// Test module for basic recommendation generation
mod recommendation_generation {
    use super::*;

    #[test]
    fn test_successful_recommendation_generation() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup scenario with forecasts
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("wheat", "grains");
        let region = factory.mock_region("US-Midwest");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        // Generate a high-demand forecast
        let (demand, confidence, hash) = oracle.generate_oracle_data(2000, 0.08); // High demand, low volatility
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        // Generate recommendation based on forecast
        let recommendation_id = client.generate_recommendation(
            &forecast_id,
            &String::from_str(&env.env, "increase_production"),
            Priority::High,
            &String::from_str(&env.env, "High demand forecasted for wheat in US-Midwest region")
        );
        
        // Verify recommendation was created
        let stored_recommendation = client.get_recommendation(&recommendation_id);
        assert_eq!(stored_recommendation.forecast_id, forecast_id);
        assert_eq!(stored_recommendation.recommendation_type, String::from_str(&env.env, "increase_production"));
        assert_eq!(stored_recommendation.priority, Priority::High);
        assert!(stored_recommendation.is_active);
    }

    #[test]
    fn test_recommendation_with_invalid_forecast() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let nonexistent_forecast_id = factory.mock_data_hash("nonexistent_forecast");
        
        let result = client.try_generate_recommendation(
            &nonexistent_forecast_id,
            &String::from_str(&env.env, "test_recommendation"),
            Priority::Medium,
            &String::from_str(&env.env, "Test description")
        );
        
        TestAssertions::assert_contract_error(result, ContractError::ForecastNotFound);
    }

    #[test]
    fn test_recommendation_types_validation() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup forecast
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("corn", "grains");
        let region = factory.mock_region("US-South");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        let (demand, confidence, hash) = oracle.generate_oracle_data(1500, 0.1);
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        // Test different recommendation types
        let recommendation_types = vec![
            "increase_production",
            "decrease_production", 
            "maintain_current",
            "switch_crops",
            "invest_in_technology",
            "seek_alternative_markets",
            "diversify_portfolio",
        ];
        
        for (i, rec_type) in recommendation_types.iter().enumerate() {
            env.advance_time(60); // Small time advancement for uniqueness
            
            let recommendation_id = client.generate_recommendation(
                &forecast_id,
                &String::from_str(&env.env, rec_type),
                Priority::Medium,
                &String::from_str(&env.env, &format!("Test recommendation for {}", rec_type))
            );
            
            let stored_recommendation = client.get_recommendation(&recommendation_id);
            assert_eq!(stored_recommendation.recommendation_type, String::from_str(&env.env, rec_type));
        }
    }

    #[test]
    fn test_recommendation_priority_levels() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup forecast
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("soybeans", "legumes");
        let region = factory.mock_region("Brazil");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        let (demand, confidence, hash) = oracle.generate_oracle_data(1200, 0.15);
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        // Test all priority levels
        let priorities = vec![Priority::Low, Priority::Medium, Priority::High, Priority::Critical];
        
        for (i, priority) in priorities.iter().enumerate() {
            env.advance_time(60);
            
            let recommendation_id = client.generate_recommendation(
                &forecast_id,
                &String::from_str(&env.env, "test_action"),
                *priority,
                &String::from_str(&env.env, &format!("Priority test {}", i))
            );
            
            let stored_recommendation = client.get_recommendation(&recommendation_id);
            assert_eq!(stored_recommendation.priority, *priority);
        }
    }
}

/// Test module for recommendation algorithms and logic
mod recommendation_algorithms {
    use super::*;

    #[test]
    fn test_high_demand_recommendation_logic() {
        let env = TestEnvironment::new();
        let client = env.client();
        let helper = IntegrationTestHelper::new(&env);
        
        // Setup scenario with high demand forecast
        let scenario = helper.setup_complete_scenario();
        let (product_id, _) = scenario.products.get(0).unwrap();
        
        // Get forecasts for the product across regions
        let forecasts = client.list_forecasts(Some(product_id.clone()), None, None, true, 0, 10);
        assert!(!forecasts.is_empty());
        
        // Generate recommendations for high-demand scenarios
        for forecast in forecasts.iter() {
            if forecast.predicted_demand > 1500 && forecast.confidence_score >= mock_data::HIGH_CONFIDENCE_THRESHOLD {
                let recommendation_id = client.generate_recommendation(
                    &forecast.forecast_id,
                    &String::from_str(&env.env, "increase_production"),
                    Priority::High,
                    &String::from_str(&env.env, "High demand with high confidence - recommend production increase")
                );
                
                let recommendation = client.get_recommendation(&recommendation_id);
                assert_eq!(recommendation.priority, Priority::High);
                assert_eq!(recommendation.recommendation_type, String::from_str(&env.env, "increase_production"));
            }
        }
    }

    #[test]
    fn test_low_confidence_recommendation_logic() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("uncertain_crop", "grains");
        let region = factory.mock_region("Volatile-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        // Generate forecast with low confidence
        let (demand, _, hash) = oracle.generate_oracle_data(1000, 0.4); // High volatility = low confidence
        let low_confidence = 45; // Below threshold
        
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            low_confidence,
            &hash
        );
        
        // Recommendation for low confidence should suggest caution
        let recommendation_id = client.generate_recommendation(
            &forecast_id,
            &String::from_str(&env.env, "maintain_current"),
            Priority::Low,
            &String::from_str(&env.env, "Low confidence forecast - recommend maintaining current strategy")
        );
        
        let recommendation = client.get_recommendation(&recommendation_id);
        assert_eq!(recommendation.priority, Priority::Low);
        assert_eq!(recommendation.recommendation_type, String::from_str(&env.env, "maintain_current"));
    }

    #[test]
    fn test_regional_diversification_recommendations() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("rice", "grains");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        // Create forecasts showing different demand across regions
        let regional_data = vec![
            ("Asia-High", 2500i128, 90u32),      // Very high demand
            ("Europe-Low", 300i128, 85u32),      // Low demand
            ("Americas-Medium", 1200i128, 88u32), // Medium demand
        ];
        
        let mut forecast_ids = Vec::new(&env.env);
        
        for (region_name, demand, confidence) in regional_data.iter() {
            env.advance_time(mock_data::MIN_FORECAST_INTERVAL + 1);
            
            let region = factory.mock_region(region_name);
            let hash = factory.mock_data_hash(&format!("{}_{}", region_name, demand));
            
            let forecast_id = client.generate_forecast(
                oracle.address(),
                &product_id,
                &region,
                *demand,
                *confidence,
                &hash
            );
            
            forecast_ids.push_back(forecast_id);
        }
        
        // Generate diversification recommendations
        let high_demand_forecast = client.get_forecast(&forecast_ids.get(0).unwrap());
        let low_demand_forecast = client.get_forecast(&forecast_ids.get(1).unwrap());
        
        // Recommend focusing on high-demand region
        let focus_recommendation_id = client.generate_recommendation(
            &high_demand_forecast.forecast_id,
            &String::from_str(&env.env, "focus_regional_expansion"),
            Priority::High,
            &String::from_str(&env.env, "Focus expansion in high-demand Asia region")
        );
        
        // Recommend reducing focus on low-demand region
        let reduce_recommendation_id = client.generate_recommendation(
            &low_demand_forecast.forecast_id,
            &String::from_str(&env.env, "reduce_regional_presence"),
            Priority::Medium,
            &String::from_str(&env.env, "Consider reducing presence in low-demand Europe region")
        );
        
        let focus_rec = client.get_recommendation(&focus_recommendation_id);
        let reduce_rec = client.get_recommendation(&reduce_recommendation_id);
        
        assert_eq!(focus_rec.priority, Priority::High);
        assert_eq!(reduce_rec.priority, Priority::Medium);
        assert!(focus_rec.priority as u8 > reduce_rec.priority as u8);
    }

    #[test]
    fn test_temporal_recommendation_patterns() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("seasonal_crop", "grains");
        let region = factory.mock_region("Seasonal-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        // Simulate seasonal demand pattern
        let seasonal_demands = vec![
            (800, "winter_low"),
            (1200, "spring_rise"),
            (1800, "summer_peak"),
            (1000, "autumn_decline"),
        ];
        
        let mut recommendations = Vec::new(&env.env);
        
        for (demand, season) in seasonal_demands.iter() {
            env.advance_time(mock_data::MIN_FORECAST_INTERVAL * 24); // Simulate longer intervals
            
            let hash = factory.mock_data_hash(season);
            let forecast_id = client.generate_forecast(
                oracle.address(),
                &product_id,
                &region,
                *demand,
                85,
                &hash
            );
            
            // Generate season-appropriate recommendations
            let rec_type = match season {
                s if s.contains("peak") => "maximize_production",
                s if s.contains("rise") => "increase_production",
                s if s.contains("decline") => "prepare_storage",
                _ => "maintain_current",
            };
            
            let priority = match demand {
                d if *d > 1500 => Priority::High,
                d if *d > 1000 => Priority::Medium,
                _ => Priority::Low,
            };
            
            let recommendation_id = client.generate_recommendation(
                &forecast_id,
                &String::from_str(&env.env, rec_type),
                priority,
                &String::from_str(&env.env, &format!("Seasonal recommendation for {}", season))
            );
            
            recommendations.push_back((recommendation_id, season.to_string()));
        }
        
        // Verify seasonal recommendations follow logical patterns
        let summer_rec = client.get_recommendation(&recommendations.get(2).unwrap().0);
        let winter_rec = client.get_recommendation(&recommendations.get(0).unwrap().0);
        
        assert_eq!(summer_rec.priority, Priority::High);
        assert_eq!(winter_rec.priority, Priority::Low);
        assert!(summer_rec.recommendation_type.to_string().contains("maximize"));
    }
}

/// Test module for recommendation accessibility and permissions
mod recommendation_accessibility {
    use super::*;

    #[test]
    fn test_recommendation_public_accessibility() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup and create recommendation
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("public_access_test", "grains");
        let region = factory.mock_region("Public-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        let (demand, confidence, hash) = oracle.generate_oracle_data(1300, 0.12);
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        let recommendation_id = client.generate_recommendation(
            &forecast_id,
            &String::from_str(&env.env, "public_recommendation"),
            Priority::Medium,
            &String::from_str(&env.env, "This recommendation should be publicly accessible")
        );
        
        // Any user should be able to access the recommendation
        let recommendation = client.get_recommendation(&recommendation_id);
        assert!(recommendation.is_active);
        assert_eq!(recommendation.recommendation_type, String::from_str(&env.env, "public_recommendation"));
    }

    #[test]
    fn test_recommendation_listing_with_filters() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("filter_test", "grains");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        // Create recommendations with different priorities
        let test_data = vec![
            ("rec_high", Priority::High),
            ("rec_medium", Priority::Medium),
            ("rec_low", Priority::Low),
            ("rec_critical", Priority::Critical),
        ];
        
        let mut recommendation_ids = Vec::new(&env.env);
        
        for (rec_type, priority) in test_data.iter() {
            env.advance_time(mock_data::MIN_FORECAST_INTERVAL + 1);
            
            let region = factory.mock_region(&format!("region_{}", rec_type));
            let (demand, confidence, hash) = oracle.generate_oracle_data(1000, 0.1);
            
            let forecast_id = client.generate_forecast(
                oracle.address(),
                &product_id,
                &region,
                demand,
                confidence,
                &hash
            );
            
            let recommendation_id = client.generate_recommendation(
                &forecast_id,
                &String::from_str(&env.env, rec_type),
                *priority,
                &String::from_str(&env.env, &format!("Test recommendation {}", rec_type))
            );
            
            recommendation_ids.push_back(recommendation_id);
        }
        
        // Test filtering by priority
        let high_priority_recs = client.list_recommendations_by_priority(Priority::High);
        assert_eq!(high_priority_recs.len(), 1);
        
        let critical_priority_recs = client.list_recommendations_by_priority(Priority::Critical);
        assert_eq!(critical_priority_recs.len(), 1);
        
        // Test filtering by product
        let product_recs = client.list_recommendations_by_product(&product_id);
        assert_eq!(product_recs.len(), 4);
    }

    #[test]
    fn test_recommendation_deactivation() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("deactivation_test", "grains");
        let region = factory.mock_region("Deactivation-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        let (demand, confidence, hash) = oracle.generate_oracle_data(1000, 0.1);
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        let recommendation_id = client.generate_recommendation(
            &forecast_id,
            &String::from_str(&env.env, "test_deactivation"),
            Priority::Medium,
            &String::from_str(&env.env, "This recommendation will be deactivated")
        );
        
        // Verify it's initially active
        let recommendation = client.get_recommendation(&recommendation_id);
        assert!(recommendation.is_active);
        
        // Deactivate the recommendation
        client.deactivate_recommendation(&recommendation_id);
        
        // Verify it's now inactive
        let deactivated_recommendation = client.get_recommendation(&recommendation_id);
        assert!(!deactivated_recommendation.is_active);
        
        // Verify it doesn't appear in active listings
        let active_recs = client.list_active_recommendations();
        let found_in_active = active_recs.iter().any(|rec| rec.recommendation_id == recommendation_id);
        assert!(!found_in_active);
    }
}

/// Test module for recommendation integration with agricultural auction contract
mod auction_integration {
    use super::*;

    #[test]
    fn test_prioritized_product_recommendations() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        // Setup multiple products with different priorities
        let oracle = OracleSimulator::new(&env.env);
        client.authorize_oracle(oracle.address());
        
        let priority_products = vec![
            ("wheat_priority", "grains", 1800i128, Priority::Critical),
            ("corn_standard", "grains", 1200i128, Priority::Medium),
            ("barley_low", "grains", 800i128, Priority::Low),
        ];
        
        let mut recommendation_ids = Vec::new(&env.env);
        
        for (product_name, category, demand, priority) in priority_products.iter() {
            let (product_id, product) = factory.create_test_product(product_name, category);
            client.register_product(&product_id, &product.name, &product.category);
            
            // Mark high-demand products as auction prioritized
            if *priority == Priority::Critical {
                client.set_auction_priority(&product_id, true);
            }
            
            env.advance_time(mock_data::MIN_FORECAST_INTERVAL + 1);
            
            let region = factory.mock_region("Auction-Region");
            let hash = factory.mock_data_hash(&format!("{}_{}", product_name, demand));
            
            let forecast_id = client.generate_forecast(
                oracle.address(),
                &product_id,
                &region,
                *demand,
                90,
                &hash
            );
            
            let recommendation_id = client.generate_recommendation(
                &forecast_id,
                &String::from_str(&env.env, "auction_recommendation"),
                *priority,
                &String::from_str(&env.env, &format!("Auction recommendation for {}", product_name))
            );
            
            recommendation_ids.push_back(recommendation_id);
        }
        
        // Test getting prioritized recommendations for auction
        let prioritized_recs = client.get_prioritized_recommendations_for_auction();
        
        // Should contain critical priority recommendations first
        assert!(!prioritized_recs.is_empty());
        let first_rec = prioritized_recs.get(0).unwrap();
        assert_eq!(first_rec.priority, Priority::Critical);
    }

    #[test]
    fn test_auction_contract_integration_signals() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("auction_signal_test", "grains");
        let region = factory.mock_region("Signal-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        // Generate high-demand forecast that should trigger auction signals
        let (demand, confidence, hash) = oracle.generate_oracle_data(2500, 0.05); // Very high demand, high confidence
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        // Generate critical recommendation that should signal auction priority
        let recommendation_id = client.generate_recommendation(
            &forecast_id,
            &String::from_str(&env.env, "urgent_auction_listing"),
            Priority::Critical,
            &String::from_str(&env.env, "Extremely high demand - recommend urgent auction listing")
        );
        
        // Verify the recommendation can be used for auction integration
        let recommendation = client.get_recommendation(&recommendation_id);
        assert_eq!(recommendation.priority, Priority::Critical);
        
        // Test auction integration signal
        let should_prioritize_in_auction = recommendation.priority == Priority::Critical 
            && demand > 2000 
            && confidence >= mock_data::HIGH_CONFIDENCE_THRESHOLD;
        
        assert!(should_prioritize_in_auction);
    }
}

/// Test module for recommendation performance and scalability
mod recommendation_performance {
    use super::*;

    #[test]
    fn test_bulk_recommendation_generation() {
        let env = TestEnvironment::new();
        let client = env.client();
        let helper = IntegrationTestHelper::new(&env);
        let perf_helper = PerformanceTestHelper::new(&env);
        
        // Setup complete scenario
        let scenario = helper.setup_complete_scenario();
        
        // Generate bulk recommendations
        let (_, execution_time) = perf_helper.measure_execution_time(|| {
            for forecast_id in scenario.forecasts.iter() {
                let recommendation_id = client.generate_recommendation(
                    forecast_id,
                    &String::from_str(&env.env, "bulk_recommendation"),
                    Priority::Medium,
                    &String::from_str(&env.env, "Bulk generated recommendation")
                );
                
                // Verify each recommendation was created
                let recommendation = client.get_recommendation(&recommendation_id);
                assert!(recommendation.is_active);
            }
        });
        
        // Performance should be reasonable
        assert!(execution_time < 5000, "Bulk recommendation generation took too long: {} ms", execution_time);
        
        // Verify all recommendations exist
        let all_recs = client.list_active_recommendations();
        assert_eq!(all_recs.len(), scenario.forecasts.len());
    }

    #[test]
    fn test_recommendation_retrieval_performance() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        let perf_helper = PerformanceTestHelper::new(&env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("perf_test", "grains");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        // Create many recommendations
        let mut recommendation_ids = Vec::new(&env.env);
        
        for i in 0..50 {
            env.advance_time(60);
            
            let region = factory.mock_region(&format!("perf_region_{}", i));
            let (demand, confidence, hash) = oracle.generate_oracle_data(1000 + (i * 10), 0.1);
            
            let forecast_id = client.generate_forecast(
                oracle.address(),
                &product_id,
                &region,
                demand,
                confidence,
                &hash
            );
            
            let recommendation_id = client.generate_recommendation(
                &forecast_id,
                &String::from_str(&env.env, "perf_recommendation"),
                Priority::Medium,
                &String::from_str(&env.env, &format!("Performance test recommendation {}", i))
            );
            
            recommendation_ids.push_back(recommendation_id);
        }
        
        // Test retrieval performance
        let (_, retrieval_time) = perf_helper.measure_execution_time(|| {
            // Retrieve all recommendations
            for recommendation_id in recommendation_ids.iter() {
                let _ = client.get_recommendation(recommendation_id);
            }
        });
        
        // Test listing performance
        let (listed_recs, listing_time) = perf_helper.measure_execution_time(|| {
            client.list_active_recommendations()
        });
        
        assert_eq!(listed_recs.len(), 50);
        assert!(retrieval_time < 3000, "Individual retrieval took too long: {} ms", retrieval_time);
        assert!(listing_time < 1000, "Listing took too long: {} ms", listing_time);
    }
}

/// Test module for edge cases and error scenarios
mod recommendation_edge_cases {
    use super::*;

    #[test]
    fn test_recommendation_with_inactive_forecast() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("inactive_test", "grains");
        let region = factory.mock_region("Inactive-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        let (demand, confidence, hash) = oracle.generate_oracle_data(1000, 0.1);
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        // Deactivate the forecast
        client.deactivate_forecast(oracle.address(), &forecast_id);
        
        // Try to create recommendation for inactive forecast
        let result = client.try_generate_recommendation(
            &forecast_id,
            &String::from_str(&env.env, "test_recommendation"),
            Priority::Medium,
            &String::from_str(&env.env, "Test description")
        );
        
        TestAssertions::assert_contract_error(result, ContractError::ForecastInactive);
    }

    #[test]
    fn test_empty_recommendation_description() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("empty_desc_test", "grains");
        let region = factory.mock_region("Empty-Desc-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        let (demand, confidence, hash) = oracle.generate_oracle_data(1000, 0.1);
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        // Test with empty description
        let empty_description = String::from_str(&env.env, "");
        let result = client.try_generate_recommendation(
            &forecast_id,
            &String::from_str(&env.env, "test_recommendation"),
            Priority::Medium,
            &empty_description
        );
        
        TestAssertions::assert_contract_error(result, ContractError::InvalidDescription);
    }

    #[test]
    fn test_duplicate_recommendation_handling() {
        let env = TestEnvironment::new();
        let client = env.client();
        let factory = TestDataFactory::new(&env.env);
        
        let oracle = OracleSimulator::new(&env.env);
        let (product_id, product) = factory.create_test_product("duplicate_test", "grains");
        let region = factory.mock_region("Duplicate-Region");
        
        client.authorize_oracle(oracle.address());
        client.register_product(&product_id, &product.name, &product.category);
        
        let (demand, confidence, hash) = oracle.generate_oracle_data(1000, 0.1);
        let forecast_id = client.generate_forecast(
            oracle.address(),
            &product_id,
            &region,
            demand,
            confidence,
            &hash
        );
        
        let recommendation_type = String::from_str(&env.env, "test_recommendation");
        let description = String::from_str(&env.env, "Test recommendation description");
        
        // Create first recommendation
        let recommendation_id1 = client.generate_recommendation(
            &forecast_id,
            &recommendation_type,
            Priority::Medium,
            &description
        );
        
        // Create similar recommendation (should be allowed but with different ID)
        let recommendation_id2 = client.generate_recommendation(
            &forecast_id,
            &recommendation_type,
            Priority::Medium,
            &description
        );
        
        // Both should exist but be unique
        assert_ne!(recommendation_id1, recommendation_id2);
        
        let rec1 = client.get_recommendation(&recommendation_id1);
        let rec2 = client.get_recommendation(&recommendation_id2);
        
        assert!(rec1.is_active);
        assert!(rec2.is_active);
        assert_ne!(rec1.recommendation_id, rec2.recommendation_id);
    }
}