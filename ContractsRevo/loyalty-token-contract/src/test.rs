#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, BytesN, Env, String, Vec,
};

use crate::{LoyaltyContract, RedemptionOption};

// Helper to set up test environment
fn setup_test() -> (Env, Address, BytesN<32>) {
    let env = Env::default();
    let contract_address = env.register(LoyaltyContract, ());
    let program_id = BytesN::from_array(&env, &[1u8; 32]);
    (env, contract_address, program_id)
}

// Helper to create a test user address
fn create_user(env: &Env) -> Address {
    Address::generate(env)
}

// Helper to create rewards
fn create_rewards(env: &Env) -> Vec<RedemptionOption> {
    let mut rewards = Vec::new(env);
    rewards.push_back(RedemptionOption {
        id: 1,
        name: String::from_str(env, "Gift Card"),
        points_required: 200,
        available_quantity: 1,
    });
    rewards.push_back(RedemptionOption {
        id: 2,
        name: String::from_str(env, "Discount Coupon"),
        points_required: 100,
        available_quantity: 2,
    });
    rewards
}

#[test]
fn test_award_points_after_transaction() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 50);
        // Points per transaction = 1, amount = 50, expect 50 points
        let points_key = (soroban_sdk::Symbol::new(&env, "points"), program_id.clone(), user.clone());
        let points: u64 = env.storage().persistent().get(&points_key).unwrap();
        assert_eq!(points, 50);
    });
}
