#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Address, Env, String, Symbol, Vec};

    fn setup_test() -> (Env, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);
        let farmer = Address::generate(&env);
        
        // Initialize contract
        PriceStabilizationContract::init(env.clone(), admin.clone());
        
        (env, admin, oracle, farmer)
    }

    #[test]
    fn test_register_chainlink_feed() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register Chainlink feed
        let result = PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_chainlink_feed_unauthorized() {
        let (env, admin, oracle, unauthorized) = setup_test();
        
        // Try to register with unauthorized address
        let result = PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            unauthorized.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_register_chainlink_feed_duplicate() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register first feed
        let result1 = PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        );
        assert!(result1.is_ok());
        
        // Try to register duplicate feed
        let result2 = PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed 2"),
        );
        
        assert!(result2.is_err());
    }

    #[test]
    fn test_update_chainlink_price() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register Chainlink feed
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Update price
        let result = PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            oracle.clone(),
            String::from_slice(&env, "corn"),
            1050, // $10.50
            env.ledger().timestamp(),
            12345,
            8,
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_chainlink_price_unauthorized_oracle() {
        let (env, admin, oracle, unauthorized_oracle) = setup_test();
        
        // Register Chainlink feed
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Try to update with unauthorized oracle
        let result = PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            unauthorized_oracle.clone(),
            String::from_slice(&env, "corn"),
            1050,
            env.ledger().timestamp(),
            12345,
            8,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_get_chainlink_price() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register Chainlink feed
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Update price
        PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            oracle.clone(),
            String::from_slice(&env, "corn"),
            1050,
            env.ledger().timestamp(),
            12345,
            8,
        ).unwrap();
        
        // Get price
        let result = PriceStabilizationContract::get_chainlink_price(
            env.clone(),
            String::from_slice(&env, "corn"),
        );
        
        assert!(result.is_ok());
        let (price, timestamp) = result.unwrap();
        assert_eq!(price, 105000000000); // Converted price
    }

    #[test]
    fn test_get_chainlink_price_unregistered_feed() {
        let (env, _, _, _) = setup_test();
        
        // Try to get price from unregistered feed
        let result = PriceStabilizationContract::get_chainlink_price(
            env.clone(),
            String::from_slice(&env, "wheat"),
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_chainlink_price_validation_stale_data() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register Chainlink feed
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Try to update with stale timestamp (2 hours ago)
        let stale_timestamp = env.ledger().timestamp() - 7200;
        let result = PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            oracle.clone(),
            String::from_slice(&env, "corn"),
            1050,
            stale_timestamp,
            12345,
            8,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_chainlink_price_validation_invalid_price() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register Chainlink feed
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Try to update with invalid price (negative)
        let result = PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            oracle.clone(),
            String::from_slice(&env, "corn"),
            -100,
            env.ledger().timestamp(),
            12345,
            8,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_chainlink_price_validation_invalid_round_id() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register Chainlink feed
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Try to update with invalid round_id (zero)
        let result = PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            oracle.clone(),
            String::from_slice(&env, "corn"),
            1050,
            env.ledger().timestamp(),
            0, // Invalid round_id
            8,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_chainlink_price_conversion() {
        let (env, admin, oracle, _) = setup_test();
        
        // Register Chainlink feed with 6 decimals
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            6,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Update price with 6 decimals
        PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            oracle.clone(),
            String::from_slice(&env, "corn"),
            1050000, // $10.50 with 6 decimals
            env.ledger().timestamp(),
            12345,
            6,
        ).unwrap();
        
        // Get price and verify conversion
        let result = PriceStabilizationContract::get_chainlink_price(
            env.clone(),
            String::from_slice(&env, "corn"),
        );
        
        assert!(result.is_ok());
        let (price, _) = result.unwrap();
        assert_eq!(price, 105000000000); // Converted to standard format
    }

    #[test]
    fn test_chainlink_integration_with_fund() {
        let (env, admin, oracle, farmer) = setup_test();
        
        // Register Chainlink feed
        PriceStabilizationContract::register_chainlink_feed(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "corn"),
            oracle.clone(),
            8,
            String::from_slice(&env, "Corn Price Feed"),
        ).unwrap();
        
        // Create stabilization fund
        let fund_id = PriceStabilizationContract::create_fund(
            env.clone(),
            admin.clone(),
            String::from_slice(&env, "Corn Fund"),
            1200, // $12.00 threshold
            String::from_slice(&env, "corn"),
        ).unwrap();
        
        // Update Chainlink price below threshold
        PriceStabilizationContract::update_chainlink_price(
            env.clone(),
            oracle.clone(),
            String::from_slice(&env, "corn"),
            1000, // $10.00 (below $12.00 threshold)
            env.ledger().timestamp(),
            12345,
            8,
        ).unwrap();
        
        // Check price threshold
        let below_threshold = PriceStabilizationContract::check_price_threshold(
            env.clone(),
            fund_id,
        ).unwrap();
        
        assert!(below_threshold); // Price should be below threshold
    }
}