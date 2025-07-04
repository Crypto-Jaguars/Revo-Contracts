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



#[test]
fn test_dynamic_point_rate_handling() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 2, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 50);
        // Points per transaction = 2, amount = 50, expect 100 points
        let points_key = (soroban_sdk::Symbol::new(&env, "points"), program_id.clone(), user.clone());
        let points: u64 = env.storage().persistent().get(&points_key).unwrap();
        assert_eq!(points, 100);
    });
}

#[test]
fn test_accurate_user_point_balances() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 100);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 50);
        // User should have 150 points
        let points_key = (soroban_sdk::Symbol::new(&env, "points"), program_id.clone(), user.clone());
        let points: u64 = env.storage().persistent().get(&points_key).unwrap();
        assert_eq!(points, 150);
    });
}


#[test]
fn test_reward_redemption_with_sufficient_points() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 200);
        // User has 200 points, redeem Gift Card (id=1, requires 200)
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 1);
        let points_key = (soroban_sdk::Symbol::new(&env, "points"), program_id.clone(), user.clone());
        let points: u64 = env.storage().persistent().get(&points_key).unwrap();
        assert_eq!(points, 0);
        // Check inventory
        let program = LoyaltyContract::get_program_info(env.clone(), program_id.clone());
        let gift_card = program.redemption_options.iter().find(|opt| opt.id == 1).unwrap();
        assert_eq!(gift_card.available_quantity, 0);
    });
}

#[test]
#[should_panic(expected = "Insufficient points")]
fn test_redemption_with_insufficient_points() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 100);
        // User has 100 points, tries to redeem Gift Card (requires 200)
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 1);
    });
}

#[test]
#[should_panic(expected = "Reward is out of stock")]
fn test_inventory_tracking_for_rewards() {
    let (env, contract_address, program_id) = setup_test();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user1.clone(), 200);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user2.clone(), 200);
        // User1 redeems Gift Card (id=1)
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user1.clone(), 1);
        // User2 tries to redeem same reward, should panic (out of stock)
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user2.clone(), 1);
    });
}

#[test]
#[should_panic]
fn test_double_redemption_attempt() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 200);
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 1);
        // Try to redeem again with 0 points, should panic
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 1);
    });
}
