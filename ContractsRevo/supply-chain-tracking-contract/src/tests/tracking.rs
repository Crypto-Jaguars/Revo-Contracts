// tests/tracking.rs
#[cfg(test)]
mod stage_tracking_tests {
    use super::*;
    use crate::{
        datatypes::{Stage, StageTier, SupplyChainError},
        tracking, product,
    };

    fn setup_test_product(env: &Env) -> (Address, BytesN<32>) {
        let farmer = create_test_farmer(env);
        let (product_type, batch_number, origin, metadata_hash) = create_test_product_data(env);

        let product_id = product::register_product(
            env.clone(),
            farmer.clone(),
            product_type,
            batch_number,
            origin,
            metadata_hash,
        ).unwrap();

        (farmer, product_id)
    }

    #[test]
    fn test_successful_stage_addition() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        let stage_id = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Seed Planting"),
            String::from_str(&env, "Field A1"),
            handler.clone(),
            data_hash,
        ).unwrap();

        assert_eq!(stage_id, 1);

        // Verify stage was added correctly
        let product = product::get_product_details(env.clone(), product_id).unwrap();
        assert_eq!(product.stages.len(), 1);

        let stage = product.stages.get(0).unwrap();
        assert_eq!(stage.stage_id, 1);
        assert_eq!(stage.tier, StageTier::Planting);
        assert_eq!(stage.name, String::from_str(&env, "Seed Planting"));
        assert_eq!(stage.location, String::from_str(&env, "Field A1"));
    }

    #[test]
    fn test_tier_progression_validation() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // First stage must be Planting
        let result = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing, // Wrong: should start with Planting
            String::from_str(&env, "Processing"),
            String::from_str(&env, "Factory"),
            handler.clone(),
            data_hash.clone(),
        );
        assert_eq!(result.unwrap_err(), SupplyChainError::InvalidTierProgression);

        // Correct progression: start with Planting
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Planting"),
            String::from_str(&env, "Field"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Next should be Processing
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing,
            String::from_str(&env, "Processing"),
            String::from_str(&env, "Factory"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Cannot skip Distribution and go to Consumer
        let result = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Consumer,
            String::from_str(&env, "Consumer"),
            String::from_str(&env, "Store"),
            handler.clone(),
            data_hash.clone(),
        );
        assert_eq!(result.unwrap_err(), SupplyChainError::InvalidTierProgression);
    }

    #[test]
    fn test_duplicate_tier_prevention() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add first stage
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Planting Stage 1"),
            String::from_str(&env, "Field A"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Attempt to add duplicate tier
        let result = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Planting Stage 2"),
            String::from_str(&env, "Field B"),
            handler.clone(),
            data_hash.clone(),
        );

        assert_eq!(result.unwrap_err(), SupplyChainError::DuplicateStageTier);
    }

    #[test]
    fn test_invalid_stage_input() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Test empty stage name
        let result1 = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, ""), // Empty name
            String::from_str(&env, "Location"),
            handler.clone(),
            data_hash.clone(),
        );
        assert_eq!(result1.unwrap_err(), SupplyChainError::InvalidInput);

        // Test empty location
        let result2 = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Stage Name"),
            String::from_str(&env, ""), // Empty location
            handler.clone(),
            data_hash.clone(),
        );
        assert_eq!(result2.unwrap_err(), SupplyChainError::InvalidInput);
    }

    #[test]
    fn test_stage_immutability() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add initial stage
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Original Stage"),
            String::from_str(&env, "Original Location"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Get original stage data
        let original_stage = tracking::get_stage_by_id(env.clone(), product_id.clone(), 1).unwrap();

        // Add another stage
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing,
            String::from_str(&env, "New Stage"),
            String::from_str(&env, "New Location"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Verify original stage hasn't changed
        let current_stage = tracking::get_stage_by_id(env.clone(), product_id.clone(), 1).unwrap();
        assert_eq!(current_stage.name, original_stage.name);
        assert_eq!(current_stage.location, original_stage.location);
        assert_eq!(current_stage.timestamp, original_stage.timestamp);
        assert_eq!(current_stage.data_hash, original_stage.data_hash);
    }

    #[test]
    fn test_sequential_stage_id_validation() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add stages and verify sequential IDs
        let stage1_id = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Stage 1"),
            String::from_str(&env, "Location 1"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();
        assert_eq!(stage1_id, 1);

        let stage2_id = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing,
            String::from_str(&env, "Stage 2"),
            String::from_str(&env, "Location 2"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();
        assert_eq!(stage2_id, 2);

        let stage3_id = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Distribution,
            String::from_str(&env, "Stage 3"),
            String::from_str(&env, "Location 3"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();
        assert_eq!(stage3_id, 3);
    }

    #[test]
    fn test_get_current_stage() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // No stages initially
        let result = tracking::get_current_stage(env.clone(), product_id.clone());
        assert_eq!(result.unwrap_err(), SupplyChainError::StageNotFound);

        // Add first stage
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "First Stage"),
            String::from_str(&env, "Location 1"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        let current = tracking::get_current_stage(env.clone(), product_id.clone()).unwrap();
        assert_eq!(current.stage_id, 1);
        assert_eq!(current.tier, StageTier::Planting);

        // Add second stage
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing,
            String::from_str(&env, "Second Stage"),
            String::from_str(&env, "Location 2"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        let current = tracking::get_current_stage(env.clone(), product_id.clone()).unwrap();
        assert_eq!(current.stage_id, 2);
        assert_eq!(current.tier, StageTier::Processing);
    }

    #[test]
    fn test_complete_supply_chain_flow() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Complete supply chain: Planting -> Processing -> Distribution -> Consumer
        let stages_data = [
            (StageTier::Planting, "Seed Planting", "Farm Field"),
            (StageTier::Processing, "Harvest & Processing", "Processing Plant"),
            (StageTier::Distribution, "Packaging & Distribution", "Distribution Center"),
            (StageTier::Consumer, "Retail Sale", "Grocery Store"),
        ];

        for (tier, name, location) in stages_data.iter() {
            tracking::add_stage(
                env.clone(),
                product_id.clone(),
                tier.clone(),
                String::from_str(&env, name),
                String::from_str(&env, location),
                handler.clone(),
                data_hash.clone(),
            ).unwrap();
        }

        // Verify complete trace
        let (product, stages) = tracking::get_product_trace(env.clone(), product_id.clone()).unwrap();
        assert_eq!(stages.len(), 4);

        // Verify final tier progression
        let current_tier = tracking::get_current_tier(env.clone(), product_id.clone()).unwrap();
        assert_eq!(current_tier, Some(StageTier::Consumer));

        // Next tier should be None (end of chain)
        let next_tier = tracking::get_next_expected_tier(env.clone(), product_id.clone()).unwrap();
        assert_eq!(next_tier, None);
    }

    #[test]
    fn test_stage_transition_validation() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add first stage
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Stage 1"),
            String::from_str(&env, "Location 1"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Valid transition 1 -> 2
        let valid = tracking::validate_stage_transition(env.clone(), product_id.clone(), 1, 2).unwrap();
        assert!(valid);

        // Invalid transition 1 -> 3 (skipping stage 2)
        let invalid = tracking::validate_stage_transition(env.clone(), product_id.clone(), 1, 3).unwrap();
        assert!(!invalid);

        // Add stage 2
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing,
            String::from_str(&env, "Stage 2"),
            String::from_str(&env, "Location 2"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Now transition 2 -> 3 should be valid
        let valid = tracking::validate_stage_transition(env.clone(), product_id.clone(), 2, 3).unwrap();
        assert!(valid);

        // Duplicate transition 1 -> 2 should fail
        let duplicate = tracking::validate_stage_transition(env.clone(), product_id.clone(), 1, 2);
        assert_eq!(duplicate.unwrap_err(), SupplyChainError::DuplicateStage);
    }

    #[test]
    fn test_stage_events_emission() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        let events_before = env.events().all().len();

        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Test Stage"),
            String::from_str(&env, "Test Location"),
            handler.clone(),
            data_hash,
        ).unwrap();

        let events = env.events().all();
        assert!(events.len() > events_before);

        // Find stage added event
        let stage_added = events.iter().find(|event| {
            event.0.as_tuple().is_some() &&
            event.0.as_tuple().unwrap().0.as_symbol() == Some(soroban_sdk::Symbol::new(&env, "stage_added"))
        });
        assert!(stage_added.is_some());
    }
}
