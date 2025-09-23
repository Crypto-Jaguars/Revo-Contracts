// tests/product.rs
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo, Events},
    Address, BytesN, Env, String, Vec,
};
use crate::{
    datatypes::{
        CertificateId, DataKey, Product, ProductRegistration, SupplyChainError,
        MAX_PRODUCTS_PER_FARMER, MAX_PRODUCTS_PER_TYPE,
    },
    product,
    utils,
};

/// Test helper to create a test environment
pub fn create_test_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    
    // Set consistent ledger timestamp for reproducible tests
    env.ledger().with_mut(|li| {
        li.timestamp = 1640995200; // Jan 1, 2022 00:00:00 UTC
    });
    
    env
}

/// Test helper to generate test data
pub fn create_test_farmer(env: &Env) -> Address {
    Address::generate(env)
}

pub fn create_test_product_data(env: &Env) -> (String, String, String, BytesN<32>) {
    (
        String::from_str(env, "Organic Tomatoes"),
        String::from_str(env, "BATCH-2024-001"),
        String::from_str(env, "Farm Valley, CA"),
        BytesN::from_array(env, &[1u8; 32]),
    )
}

#[cfg(test)]
mod product_registration_tests {
    use super::*;

    #[test]
    fn test_successful_product_registration() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);
        let (product_type, batch_number, origin, metadata_hash) = create_test_product_data(&env);

        let result = product::register_product(
            env.clone(),
            farmer.clone(),
            product_type.clone(),
            batch_number.clone(),
            origin.clone(),
            metadata_hash.clone(),
        );

        assert!(result.is_ok());
        let product_id = result.unwrap();

        // Verify product was stored correctly
        let stored_product = product::get_product_details(env.clone(), product_id.clone());
        assert!(stored_product.is_ok());
        
        let product_data = stored_product.unwrap();
        assert_eq!(product_data.farmer_id, farmer);
        assert_eq!(product_data.product_id, product_id);
        assert!(product_data.stages.is_empty());
        assert_eq!(product_data.certificate_id, CertificateId::None);

        // Verify registration details were stored
        let registration = product::get_product_registration(env.clone(), product_id.clone());
        assert!(registration.is_ok());
        
        let reg_data = registration.unwrap();
        assert_eq!(reg_data.product_type, product_type);
        assert_eq!(reg_data.batch_number, batch_number);
        assert_eq!(reg_data.origin_location, origin);
        assert_eq!(reg_data.metadata_hash, metadata_hash);
    }

    #[test]
    fn test_duplicate_product_registration_fails() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);
        let (product_type, batch_number, origin, metadata_hash) = create_test_product_data(&env);

        // First registration should succeed
        let result1 = product::register_product(
            env.clone(),
            farmer.clone(),
            product_type.clone(),
            batch_number.clone(),
            origin.clone(),
            metadata_hash.clone(),
        );
        assert!(result1.is_ok());

        // Second registration with same data should fail
        let result2 = product::register_product(
            env.clone(),
            farmer.clone(),
            product_type.clone(),
            batch_number.clone(),
            origin.clone(),
            metadata_hash.clone(),
        );
        
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), SupplyChainError::DuplicateProduct);
    }

    #[test]
    fn test_invalid_input_data_fails() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);
        let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

        // Test empty product type
        let result1 = product::register_product(
            env.clone(),
            farmer.clone(),
            String::from_str(&env, ""),
            String::from_str(&env, "BATCH-001"),
            String::from_str(&env, "Location"),
            metadata_hash.clone(),
        );
        assert_eq!(result1.unwrap_err(), SupplyChainError::InvalidInput);

        // Test empty batch number
        let result2 = product::register_product(
            env.clone(),
            farmer.clone(),
            String::from_str(&env, "Tomatoes"),
            String::from_str(&env, ""),
            String::from_str(&env, "Location"),
            metadata_hash.clone(),
        );
        assert_eq!(result2.unwrap_err(), SupplyChainError::InvalidInput);

        // Test empty origin location
        let result3 = product::register_product(
            env.clone(),
            farmer.clone(),
            String::from_str(&env, "Tomatoes"),
            String::from_str(&env, "BATCH-001"),
            String::from_str(&env, ""),
            metadata_hash.clone(),
        );
        assert_eq!(result3.unwrap_err(), SupplyChainError::InvalidInput);
    }

    #[test]
    fn test_farmer_product_limit_exceeded() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);
        let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

        // Register maximum allowed products
        for i in 0..MAX_PRODUCTS_PER_FARMER {
            let result = product::register_product(
                env.clone(),
                farmer.clone(),
                String::from_str(&env, "Tomatoes"),
                String::from_str(&env, &format!("BATCH-{:03}", i)),
                String::from_str(&env, "Farm Location"),
                metadata_hash.clone(),
            );
            assert!(result.is_ok(), "Failed at product {}", i);
        }

        // Next registration should fail due to limit
        let result = product::register_product(
            env.clone(),
            farmer.clone(),
            String::from_str(&env, "Tomatoes"),
            String::from_str(&env, "BATCH-OVERFLOW"),
            String::from_str(&env, "Farm Location"),
            metadata_hash.clone(),
        );
        
        assert_eq!(result.unwrap_err(), SupplyChainError::ProductLimitExceeded);
    }

    #[test]
    fn test_product_type_limit_exceeded() {
        let env = create_test_env();
        let product_type = String::from_str(&env, "Limited Type");
        let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

        // Register maximum allowed products of same type with different farmers
        for i in 0..MAX_PRODUCTS_PER_TYPE {
            let farmer = Address::generate(&env);
            let result = product::register_product(
                env.clone(),
                farmer,
                product_type.clone(),
                String::from_str(&env, &format!("BATCH-{:03}", i)),
                String::from_str(&env, "Farm Location"),
                metadata_hash.clone(),
            );
            assert!(result.is_ok(), "Failed at product type index {}", i);
        }

        // Next registration of same type should fail
        let farmer = Address::generate(&env);
        let result = product::register_product(
            env.clone(),
            farmer,
            product_type.clone(),
            String::from_str(&env, "BATCH-OVERFLOW"),
            String::from_str(&env, "Farm Location"),
            metadata_hash.clone(),
        );
        
        assert_eq!(result.unwrap_err(), SupplyChainError::ProductLimitExceeded);
    }

    #[test]
    fn test_list_products_by_farmer() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);
        let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

        // Register multiple products
        let mut expected_ids = Vec::new(&env);
        for i in 0..3 {
            let product_id = product::register_product(
                env.clone(),
                farmer.clone(),
                String::from_str(&env, "Tomatoes"),
                String::from_str(&env, &format!("BATCH-{:03}", i)),
                String::from_str(&env, "Farm Location"),
                metadata_hash.clone(),
            ).unwrap();
            expected_ids.push_back(product_id);
        }

        // Test listing products by farmer
        let farmer_products = product::list_products_by_farmer(env.clone(), farmer.clone()).unwrap();
        assert_eq!(farmer_products.len(), 3);

        // Verify all products are included
        for expected_id in expected_ids.iter() {
            assert!(farmer_products.iter().any(|id| *id == expected_id));
        }
    }

    #[test]
    fn test_list_products_by_type() {
        let env = create_test_env();
        let product_type = String::from_str(&env, "Organic Apples");
        let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

        // Register multiple products of same type
        let mut expected_ids = Vec::new(&env);
        for i in 0..3 {
            let farmer = Address::generate(&env);
            let product_id = product::register_product(
                env.clone(),
                farmer,
                product_type.clone(),
                String::from_str(&env, &format!("BATCH-{:03}", i)),
                String::from_str(&env, "Farm Location"),
                metadata_hash.clone(),
            ).unwrap();
            expected_ids.push_back(product_id);
        }

        // Test listing products by type
        let type_products = product::list_products_by_type(env.clone(), product_type.clone()).unwrap();
        assert_eq!(type_products.len(), 3);

        // Verify all products are included
        for expected_id in expected_ids.iter() {
            assert!(type_products.iter().any(|id| *id == expected_id));
        }
    }

    #[test]
    fn test_product_registration_events() {
        let env = create_test_env();
        let farmer = create_test_farmer(&env);
        let (product_type, batch_number, origin, metadata_hash) = create_test_product_data(&env);

        // Clear existing events
        let events_before = env.events().all().len();

        let product_id = product::register_product(
            env.clone(),
            farmer.clone(),
            product_type,
            batch_number,
            origin,
            metadata_hash,
        ).unwrap();

        // Verify events were emitted
        let events = env.events().all();
        assert!(events.len() > events_before);

        // Find product registration event
        let product_registered = events.iter().find(|event| {
            event.0.as_tuple().is_some() &&
            event.0.as_tuple().unwrap().0.as_symbol() == Some(soroban_sdk::Symbol::new(&env, "product_registered"))
        });
        assert!(product_registered.is_some());
    }

    #[test]
    fn test_concurrent_product_registrations() {
        let env = create_test_env();
        let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

        // Simulate concurrent registrations from different farmers
        let mut product_ids = Vec::new(&env);
        for i in 0..5 {
            let farmer = Address::generate(&env);
            let product_id = product::register_product(
                env.clone(),
                farmer,
                String::from_str(&env, &format!("Product-Type-{}", i)),
                String::from_str(&env, &format!("BATCH-{:03}", i)),
                String::from_str(&env, "Farm Location"),
                metadata_hash.clone(),
            ).unwrap();
            product_ids.push_back(product_id);
        }

        // Verify all products have unique IDs
        for i in 0..product_ids.len() {
            for j in (i + 1)..product_ids.len() {
                assert_ne!(product_ids.get(i).unwrap(), product_ids.get(j).unwrap());
            }
        }
    }

    #[test]
    fn test_product_not_found_error() {
        let env = create_test_env();
        let non_existent_id = BytesN::from_array(&env, &[0u8; 32]);

        let result = product::get_product_details(env.clone(), non_existent_id.clone());
        assert_eq!(result.unwrap_err(), SupplyChainError::ProductNotFound);

        let registration_result = product::get_product_registration(env.clone(), non_existent_id);
        assert_eq!(registration_result.unwrap_err(), SupplyChainError::ProductNotFound);
    }
}