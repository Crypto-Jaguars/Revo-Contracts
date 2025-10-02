use crate::{CSAMembershipContract, CSAMembershipContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String,
};

pub struct TestEnv {
    pub env: Env,
    pub contract_id: Address,
    pub admin: Address,
    pub member1: Address,
    pub member2: Address,
    pub member3: Address,
}

pub fn setup_test() -> TestEnv {
    let env = Env::default();
    env.ledger().with_mut(|li| {
        li.timestamp = 1700000000; // Set current time to Nov 2023
    });

    let contract_id = env.register(CSAMembershipContract, ());
    let admin = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);

    TestEnv {
        env,
        contract_id,
        admin,
        member1,
        member2,
        member3,
    }
}

pub fn create_client(test_env: &TestEnv) -> CSAMembershipContractClient {
    CSAMembershipContractClient::new(&test_env.env, &test_env.contract_id)
}

pub fn create_farm_id(env: &Env, seed: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    BytesN::from_array(env, &bytes)
}

pub fn standard_farm_id(env: &Env) -> BytesN<32> {
    create_farm_id(env, 1)
}

pub fn standard_season(env: &Env) -> String {
    String::from_str(env, "Summer 2025")
}

pub fn standard_pickup_location(env: &Env) -> String {
    String::from_str(env, "Downtown Market")
}

pub const FUTURE_START_DATE: u64 = 1735689600; // Jan 1, 2025
pub const FUTURE_END_DATE: u64 = 1743465600; // Apr 1, 2025
pub const SUBSCRIPTION_AMOUNT: i128 = 10_000_000; // 10 tokens

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_test() {
        let test_env = setup_test();
        assert!(test_env.env.ledger().timestamp() == 1700000000);
    }

    #[test]
    fn test_create_farm_id() {
        let test_env = setup_test();
        let farm_id1 = create_farm_id(&test_env.env, 1);
        let farm_id2 = create_farm_id(&test_env.env, 2);
        assert!(farm_id1 != farm_id2);
    }

    #[test]
    fn test_standard_farm_id() {
        let test_env = setup_test();
        let farm_id = standard_farm_id(&test_env.env);
        let expected = create_farm_id(&test_env.env, 1);
        assert_eq!(farm_id, expected);
    }

    #[test]
    fn test_standard_season() {
        let test_env = setup_test();
        let season = standard_season(&test_env.env);
        assert_eq!(season, String::from_str(&test_env.env, "Summer 2025"));
    }

    #[test]
    fn test_create_client() {
        let test_env = setup_test();
        let _client = create_client(&test_env);
        // Test passes if client creation doesn't panic
    }
}
