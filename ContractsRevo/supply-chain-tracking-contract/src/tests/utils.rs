#[cfg(test)]
mod utils_tests {
    use super::*;
    use crate::{
        datatypes::{DataKey, SupplyChainError},
        utils, tracking, product,
    };

    #[test]
    fn test_generate_product_id_uniqueness() {
        let env = create_test_env();
        let farmer1 = Address::generate(&env);
        let farmer2 = Address::generate(&env);
        let product_type = String::from_str(&env, "Tomatoes");
        let batch_number = String::from_str(&env, "BATCH-001");

        let id1 = utils::generate_product_id(&env, &farmer1, &product_type, &batch_number);
        let id2 = utils::generate_product_id(&env, &farmer2, &product_type, &batch_number);

        // Different farmers should generate different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_product_id_deterministic() {
        let env = create_test_env();
        let farmer = Address::generate(&env);
        let product_type = String::from_str(&env, "Tomatoes");
        let batch_number = String::from_str(&env, "BATCH-001");

        // Same inputs should generate same ID (within same timestamp)
        let id1 = utils::generate_product_id(&env, &farmer, &product_type, &batch_number);
        let id2 = utils::generate_product_id(&env, &farmer, &product_type, &batch_number);

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_calculate_supply_chain_hash() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add stages
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            crate::datatypes::StageTier::Planting,
            String::from_str(&env, "Planting"),
            String::from_str(&env, "Field"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            crate::datatypes::StageTier::Processing,
            String::from_str(&env, "Processing"),
            String::from_str(&env, "Factory"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        let result = utils::calculate_supply_chain_hash(&env, &product_id);
        assert!(result.is_ok());

        let hash = result.unwrap();
        // Hash should not be all zeros
        assert_ne!(hash.to_array(), [0u8; 32]);
    }

    #[test]
    fn test_calculate_supply_chain_hash_empty_stages() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);

        let result = utils::calculate_supply_chain_hash(&env, &product_id);
        assert_eq!(result.unwrap_err(), SupplyChainError::InvalidHash);
    }

    #[test]
    fn test_calculate_supply_chain_hash_consistency() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add same stages
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            crate::datatypes::StageTier::Planting,
            String::from_str(&env, "Planting"),
            String::from_str(&env, "Field"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        let hash1 = utils::calculate_supply_chain_hash(&env, &product_id).unwrap();
        let hash2 = utils::calculate_supply_chain_hash(&env, &product_id).unwrap();

        // Should be consistent
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_generate_qr_code_data() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);

        let qr_result = utils::generate_qr_code_data(&env, &product_id);
        assert!(qr_result.is_ok());

        let qr_code = qr_result.unwrap();
        assert!(!qr_code.is_empty());

        // Should be able to resolve back to product ID
        let resolved = utils::resolve_qr_code(&env, &qr_code);
        assert!(resolved.is_ok());
        assert_eq!(resolved.unwrap(), product_id);
    }

    #[test]
    fn test_qr_code_collision_resistance() {
        let env = create_test_env();
        let mut qr_codes = Vec::new(&env);

        // Generate multiple QR codes and check for uniqueness
        for i in 0..10 {
            let farmer = Address::generate(&env);
            let product_id = product::register_product(
                env.clone(),
                farmer,
                String::from_str(&env, "Product"),
                String::from_str(&env, &format!("BATCH-{:03}", i)),
                String::from_str(&env, "Location"),
                BytesN::from_array(&env, &[1u8; 32]),
            ).unwrap();

            let qr_code = utils::generate_qr_code_data(&env, &product_id).unwrap();
            qr_codes.push_back(qr_code);
        }

        // Verify all QR codes are unique
        for i in 0..qr_codes.len() {
            for j in (i + 1)..qr_codes.len() {
                assert_ne!(qr_codes.get(i).unwrap(), qr_codes.get(j).unwrap());
            }
        }
    }

    #[test]
    fn test_resolve_qr_code_not_found() {
        let env = create_test_env();
        let invalid_qr = String::from_str(&env, "invalid_qr_code");

        let result = utils::resolve_qr_code(&env, &invalid_qr);
        assert_eq!(result.unwrap_err(), SupplyChainError::QRCodeNotFound);
    }

    #[test]
    fn test_verify_hash_chain_integrity() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add stages
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            crate::datatypes::StageTier::Planting,
            String::from_str(&env, "Planting"),
            String::from_str(&env, "Field"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            crate::datatypes::StageTier::Processing,
            String::from_str(&env, "Processing"),
            String::from_str(&env, "Factory"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        let result = utils::verify_hash_chain(&env, &product_id);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_hash_chain_empty_product() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);

        let result = utils::verify_hash_chain(&env, &product_id);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false for empty stages
    }

    #[test]
    fn test_convert_bytes_to_u32_deterministic() {
        let env = create_test_env();
        let cert_bytes = BytesN::from_array(&env, &[1u8; 32]);

        let result1 = utils::convert_bytes_to_u32(&env, &cert_bytes);
        let result2 = utils::convert_bytes_to_u32(&env, &cert_bytes);

        // Should be deterministic
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_convert_bytes_to_u32_different_inputs() {
        let env = create_test_env();
        let cert_bytes1 = BytesN::from_array(&env, &[1u8; 32]);
        let cert_bytes2 = BytesN::from_array(&env, &[2u8; 32]);

        let result1 = utils::convert_bytes_to_u32(&env, &cert_bytes1);
        let result2 = utils::convert_bytes_to_u32(&env, &cert_bytes2);

        // Different inputs should produce different outputs
        assert_ne!(result1, result2);
    }
}

// tests/integration.rs
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{
        datatypes::{CertificateId, StageTier, SupplyChainError},
        product, tracking, validation, utils,
    };

    #[test]
    fn test_complete_supply_chain_lifecycle() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);
        let (product_type, batch_number, origin, metadata_hash) = create_test_product_data(&env);

        // 1. Register product
        let product_id = product::register_product(
            env.clone(),
            farmer.clone(),
            product_type.clone(),
            batch_number.clone(),
            origin.clone(),
            metadata_hash.clone(),
        ).unwrap();

        // 2. Add complete supply chain stages
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        let stages_data = [
            (StageTier::Planting, "Seed Planting", "Farm Field A"),
            (StageTier::Processing, "Harvest & Processing", "Processing Plant"),
            (StageTier::Distribution, "Packaging & Shipping", "Distribution Center"),
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

        // 3. Generate QR code for consumer access
        let qr_code = utils::generate_qr_code_data(&env, &product_id).unwrap();

        // 4. Verify complete supply chain
        let supply_chain_hash = utils::calculate_supply_chain_hash(&env, &product_id).unwrap();
        let authenticity_check = validation::verify_authenticity(
            env.clone(),
            farmer.clone(),
            product_id.clone(),
            supply_chain_hash,
        ).unwrap();

        assert!(authenticity_check);

        // 5. Verify hash chain integrity
        let hash_chain_valid = utils::verify_hash_chain(&env, &product_id).unwrap();
        assert!(hash_chain_valid);

        // 6. Test consumer access via QR code
        let resolved_product_id = utils::resolve_qr_code(&env, &qr_code).unwrap();
        assert_eq!(resolved_product_id, product_id);

        // 7. Get complete product trace
        let (final_product, all_stages) = tracking::get_product_trace(env.clone(), product_id.clone()).unwrap();
        assert_eq!(all_stages.len(), 4);
        assert_eq!(final_product.farmer_id, farmer);
    }

    #[test]
    fn test_high_volume_product_registration() {
        let env = create_test_env();
        let batch_size = 50; // Test scalability
        let mut product_ids = Vec::new(&env);

        // Register multiple products from different farmers
        for i in 0..batch_size {
            let farmer = Address::generate(&env);
            let product_id = product::register_product(
                env.clone(),
                farmer,
                String::from_str(&env, &format!("Product-Type-{}", i % 5)), // 5 different types
                String::from_str(&env, &format!("BATCH-{:04}", i)),
                String::from_str(&env, "Farm Location"),
                BytesN::from_array(&env, &[(i % 256) as u8; 32]),
            ).unwrap();
            
            product_ids.push_back(product_id);
        }

        // Verify all products are unique and accessible
        assert_eq!(product_ids.len() as u32, batch_size);
        
        for (i, product_id) in product_ids.iter().enumerate() {
            let product = product::get_product_details(env.clone(), product_id).unwrap();
            assert_eq!(product.product_id, product_id);
            
            // Verify unique IDs
            for j in ((i + 1) as u32)..batch_size {
                assert_ne!(product_id, product_ids.get(j).unwrap());
            }
        }
    }

    #[test] 
    fn test_concurrent_stage_additions() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);
        
        // Simulate rapid stage additions
        let handlers = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];
        
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);
        
        // Add stages rapidly
        let stage1_id = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Planting Stage"),
            String::from_str(&env, "Field"),
            handlers[0].clone(),
            data_hash.clone(),
        ).unwrap();

        let stage2_id = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing,
            String::from_str(&env, "Processing Stage"),
            String::from_str(&env, "Factory"),
            handlers[1].clone(),
            data_hash.clone(),
        ).unwrap();

        let stage3_id = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Distribution,
            String::from_str(&env, "Distribution Stage"),
            String::from_str(&env, "Warehouse"),
            handlers[2].clone(),
            data_hash.clone(),
        ).unwrap();

        // Verify sequential stage IDs
        assert_eq!(stage1_id, 1);
        assert_eq!(stage2_id, 2);
        assert_eq!(stage3_id, 3);

        // Verify all stages are properly stored
        let product = product::get_product_details(env.clone(), product_id).unwrap();
        assert_eq!(product.stages.len(), 3);
    }

    #[test]
    fn test_boundary_conditions() {
        let env = create_test_env();

        // Test maximum products per farmer
        let farmer = create_test_farmer(&env);
        let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

        // Register maximum allowed products
        for i in 0..MAX_PRODUCTS_PER_FARMER {
            let result = product::register_product(
                env.clone(),
                farmer.clone(),
                String::from_str(&env, "Test Product"),
                String::from_str(&env, &format!("BATCH-{:03}", i)),
                String::from_str(&env, "Test Location"),
                metadata_hash.clone(),
            );
            assert!(result.is_ok(), "Failed to register product {} for farmer", i);
        }

        // Next registration should fail
        let overflow_result = product::register_product(
            env.clone(),
            farmer.clone(),
            String::from_str(&env, "Test Product"),
            String::from_str(&env, "BATCH-OVERFLOW"),
            String::from_str(&env, "Test Location"),
            metadata_hash.clone(),
        );
        assert_eq!(overflow_result.unwrap_err(), SupplyChainError::ProductLimitExceeded);

        // Test maximum products per type
        let product_type = String::from_str(&env, "Limited Type Product");
        
        // Register maximum allowed products of same type
        for i in 0..MAX_PRODUCTS_PER_TYPE {
            let new_farmer = Address::generate(&env);
            let result = product::register_product(
                env.clone(),
                new_farmer,
                product_type.clone(),
                String::from_str(&env, &format!("TYPE-BATCH-{:03}", i)),
                String::from_str(&env, "Test Location"),
                metadata_hash.clone(),
            );
            assert!(result.is_ok(), "Failed to register product type {} at index {}", product_type, i);
        }

        // Next registration of same type should fail
        let type_overflow_farmer = Address::generate(&env);
        let type_overflow_result = product::register_product(
            env.clone(),
            type_overflow_farmer,
            product_type.clone(),
            String::from_str(&env, "TYPE-BATCH-OVERFLOW"),
            String::from_str(&env, "Test Location"),
            metadata_hash.clone(),
        );
        assert_eq!(type_overflow_result.unwrap_err(), SupplyChainError::ProductLimitExceeded);
    }

    #[test]
    fn test_data_consistency_under_stress() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);

        // Create multiple products and verify data consistency
        let num_products = 20;
        let mut products = Vec::new(&env);

        for i in 0..num_products {
            let product_id = product::register_product(
                env.clone(),
                farmer.clone(),
                String::from_str(&env, &format!("Product-{}", i)),
                String::from_str(&env, &format!("BATCH-{:04}", i)),
                String::from_str(&env, "Consistent Location"),
                BytesN::from_array(&env, &[(i % 256) as u8; 32]),
            ).unwrap();

            products.push_back(product_id);

            // Add stages to each product
            let handler = Address::generate(&env);
            let data_hash = BytesN::from_array(&env, &[(i % 256) as u8; 32]);

            tracking::add_stage(
                env.clone(),
                product_id.clone(),
                StageTier::Planting,
                String::from_str(&env, &format!("Planting-{}", i)),
                String::from_str(&env, &format!("Field-{}", i)),
                handler,
                data_hash,
            ).unwrap();
        }

        // Verify all products maintain data consistency
        for (i, product_id) in products.iter().enumerate() {
            let product = product::get_product_details(env.clone(), product_id).unwrap();
            assert_eq!(product.farmer_id, farmer);
            assert_eq!(product.stages.len(), 1);

            let registration = product::get_product_registration(env.clone(), product_id).unwrap();
            assert_eq!(registration.product_type, String::from_str(&env, &format!("Product-{}", i)));
            assert_eq!(registration.batch_number, String::from_str(&env, &format!("BATCH-{:04}", i)));

            // Verify hash chain integrity
            let hash_chain_valid = utils::verify_hash_chain(&env, &product_id).unwrap();
            assert!(hash_chain_valid);
        }
    }

    #[test]
    fn test_error_recovery_scenarios() {
        let env = create_test_env();
        let (farmer, product_id) = setup_test_product(&env);

        // Test recovery from invalid operations
        let handler = Address::generate(&env);
        let data_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Add valid first stage
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Planting,
            String::from_str(&env, "Valid Stage"),
            String::from_str(&env, "Valid Location"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        // Attempt invalid operations
        let invalid_tier_result = tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Consumer, // Skip tiers
            String::from_str(&env, "Invalid Stage"),
            String::from_str(&env, "Invalid Location"),
            handler.clone(),
            data_hash.clone(),
        );
        assert_eq!(invalid_tier_result.unwrap_err(), SupplyChainError::InvalidTierProgression);

        // Verify system state is still consistent after error
        let product = product::get_product_details(env.clone(), product_id.clone()).unwrap();
        assert_eq!(product.stages.len(), 1); // Only valid stage should exist

        // Continue with valid operations
        tracking::add_stage(
            env.clone(),
            product_id.clone(),
            StageTier::Processing,
            String::from_str(&env, "Recovery Stage"),
            String::from_str(&env, "Recovery Location"),
            handler.clone(),
            data_hash.clone(),
        ).unwrap();

        let final_product = product::get_product_details(env.clone(), product_id).unwrap();
        assert_eq!(final_product.stages.len(), 2); // Now should have 2 stages
    }