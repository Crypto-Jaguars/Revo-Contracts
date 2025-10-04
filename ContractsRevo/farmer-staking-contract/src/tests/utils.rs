use crate::utils::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, BytesN, Env,
};

/// Test helper to create a test environment
pub fn create_test_env() -> Env {
    Env::default()
}

/// Test helper to create test addresses
pub fn create_test_addresses(env: &Env) -> (Address, Address, Address) {
    let admin = Address::generate(env);
    let farmer = Address::generate(env);
    let token_address = Address::generate(env);
    (admin, farmer, token_address)
}

/// Test helper to set up time
pub fn setup_time(env: &Env, timestamp: u64) {
    env.ledger().with_mut(|li| {
        li.timestamp = timestamp;
    });
}

/// Test helper to create a fake pool ID
pub fn create_fake_pool_id(env: &Env) -> BytesN<32> {
    let data = Bytes::from_array(env, &[1u8; 32]);
    env.crypto().sha256(&data).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(1000, 10), 100);
        assert_eq!(calculate_percentage(5000, 5), 250);
        assert_eq!(calculate_percentage(100, 50), 50);
        assert_eq!(calculate_percentage(0, 10), 0);
    }

    #[test]
    fn test_seconds_to_days() {
        assert_eq!(seconds_to_days(86400), 1);
        assert_eq!(seconds_to_days(172800), 2);
        assert_eq!(seconds_to_days(604800), 7);
        assert_eq!(seconds_to_days(0), 0);
    }

    #[test]
    fn test_days_to_seconds() {
        assert_eq!(days_to_seconds(1), 86400);
        assert_eq!(days_to_seconds(7), 604800);
        assert_eq!(days_to_seconds(30), 2592000);
        assert_eq!(days_to_seconds(0), 0);
    }

    #[test]
    fn test_time_elapsed() {
        assert_eq!(time_elapsed(100, 200), 100);
        assert_eq!(time_elapsed(200, 100), 0);
        assert_eq!(time_elapsed(100, 100), 0);
        assert_eq!(time_elapsed(0, 1000), 1000);
    }

    #[test]
    fn test_safe_add() {
        assert_eq!(safe_add(100, 200).unwrap(), 300);
        assert_eq!(safe_add(0, 0).unwrap(), 0);
        assert_eq!(safe_add(-100, 200).unwrap(), 100);
    }

    #[test]
    fn test_safe_sub() {
        assert_eq!(safe_sub(200, 100).unwrap(), 100);
        assert_eq!(safe_sub(100, 100).unwrap(), 0);
        assert_eq!(safe_sub(100, 200).unwrap(), -100);
    }

    #[test]
    fn test_safe_mul() {
        assert_eq!(safe_mul(10, 20).unwrap(), 200);
        assert_eq!(safe_mul(0, 100).unwrap(), 0);
        assert_eq!(safe_mul(-10, 20).unwrap(), -200);
    }

    #[test]
    fn test_safe_div() {
        assert_eq!(safe_div(100, 10).unwrap(), 10);
        assert_eq!(safe_div(100, 3).unwrap(), 33);
        assert!(safe_div(100, 0).is_err());
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount(100).is_ok());
        assert!(validate_amount(1).is_ok());
        assert!(validate_amount(0).is_err());
        assert!(validate_amount(-1).is_err());
    }

    #[test]
    fn test_validate_lock_period() {
        assert!(validate_lock_period(100, 1000).is_ok());
        assert!(validate_lock_period(1000, 1000).is_ok());
        assert!(validate_lock_period(1001, 1000).is_err());
    }

    #[test]
    fn test_is_time_past() {
        let env = create_test_env();
        setup_time(&env, 1000);

        assert!(is_time_past(&env, 999));
        assert!(is_time_past(&env, 1000));
        assert!(!is_time_past(&env, 1001));
    }

    #[test]
    fn test_get_current_timestamp() {
        let env = create_test_env();
        setup_time(&env, 12345);

        assert_eq!(get_current_timestamp(&env), 12345);
    }

    #[test]
    fn test_create_test_addresses() {
        let env = create_test_env();
        let (admin, farmer, token) = create_test_addresses(&env);

        // Addresses should be different
        assert_ne!(admin, farmer);
        assert_ne!(admin, token);
        assert_ne!(farmer, token);
    }
}
