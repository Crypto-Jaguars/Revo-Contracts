//! Test utilities and helper functions for environmental-impact-tracking-contract
//!
//! This module provides common setup functions, data generators, and assertion helpers
//! used across all test modules.

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::EnvironmentalContract;

/// Test environment setup with contract and addresses
pub struct TestEnv {
    pub env: Env,
    pub contract_id: Address,
    pub admin: Address,
    pub user1: Address,
    pub user2: Address,
}

/// Sets up a fresh test environment with registered contract and generated addresses
pub fn setup_test() -> TestEnv {
    let env = Env::default();
    let contract_id = env.register(EnvironmentalContract, ());
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    TestEnv {
        env,
        contract_id,
        admin,
        user1,
        user2,
    }
}

/// Generates a unique credit ID from a seed value
pub fn create_credit_id(env: &Env, seed: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    BytesN::from_array(env, &bytes)
}

/// Generates a unique project ID from a seed value
pub fn create_project_id(env: &Env, seed: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[1] = seed; // Use different byte position to avoid conflicts
    BytesN::from_array(env, &bytes)
}

/// Creates a standard verification method string
pub fn standard_verification_method(env: &Env) -> String {
    String::from_str(env, "Verified Carbon Standard")
}

/// Creates an alternative verification method string for testing
pub fn alternative_verification_method(env: &Env) -> String {
    String::from_str(env, "Gold Standard")
}

/// Creates a custom verification method string
pub fn custom_verification_method(env: &Env, method: &str) -> String {
    String::from_str(env, method)
}

/// Standard carbon amount for testing (1000 kg)
pub const STANDARD_CARBON_AMOUNT: u32 = 1000;

/// Large carbon amount for testing (100,000 kg)
pub const LARGE_CARBON_AMOUNT: u32 = 100_000;

/// Maximum allowed carbon amount
pub const MAX_CARBON_AMOUNT: u32 = 1_000_000_000;

/// Generates multiple credit IDs for batch testing
pub fn generate_credit_ids(env: &Env, count: u8) -> soroban_sdk::Vec<BytesN<32>> {
    let mut credits = soroban_sdk::Vec::new(env);
    for i in 0..count {
        credits.push_back(create_credit_id(env, i + 1));
    }
    credits
}

/// Generates multiple project IDs for batch testing
pub fn generate_project_ids(env: &Env, count: u8) -> soroban_sdk::Vec<BytesN<32>> {
    let mut projects = soroban_sdk::Vec::new(env);
    for i in 0..count {
        projects.push_back(create_project_id(env, i + 1));
    }
    projects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_creates_fresh_env() {
        let test_env = setup_test();
        assert!(test_env.env.ledger().timestamp() >= 0);
    }

    #[test]
    fn test_credit_ids_are_unique() {
        let env = Env::default();
        let id1 = create_credit_id(&env, 1);
        let id2 = create_credit_id(&env, 2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_project_ids_are_unique() {
        let env = Env::default();
        let id1 = create_project_id(&env, 1);
        let id2 = create_project_id(&env, 2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_credit_and_project_ids_differ() {
        let env = Env::default();
        let credit_id = create_credit_id(&env, 1);
        let project_id = create_project_id(&env, 1);
        assert_ne!(credit_id, project_id);
    }

    #[test]
    fn test_generate_multiple_credit_ids() {
        let env = Env::default();
        let credits = generate_credit_ids(&env, 5);
        assert_eq!(credits.len(), 5);

        // Verify uniqueness
        for i in 0..5 {
            for j in (i + 1)..5 {
                assert_ne!(credits.get(i).unwrap(), credits.get(j).unwrap());
            }
        }
    }
}
