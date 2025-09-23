#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

use crate::{LoyaltyContract, LoyaltyProgram, RedemptionOption};

// ============ CORE SETUP FUNCTIONS ============

pub fn setup_test() -> (Env, Address, BytesN<32>) {
    let env = Env::default();
    let contract_address = env.register(LoyaltyContract, ());
    let program_id = BytesN::from_array(&env, &[1u8; 32]);
    (env, contract_address, program_id)
}

pub fn create_user(env: &Env) -> Address {
    Address::generate(env)
}

pub fn create_multiple_users(env: &Env, count: u32) -> Vec<Address> {
    let mut users = Vec::new(env);
    for _ in 0..count {
        users.push_back(Address::generate(env));
    }
    users
}

// ============ REWARD CREATION FUNCTIONS ============

pub fn create_rewards(env: &Env) -> Vec<RedemptionOption> {
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

pub fn create_basic_rewards(env: &Env) -> Vec<RedemptionOption> {
    let mut rewards = Vec::new(env);
    rewards.push_back(RedemptionOption {
        id: 1,
        name: String::from_str(env, "Gift Card"),
        points_required: 200,
        available_quantity: 10,
    });
    rewards.push_back(RedemptionOption {
        id: 2,
        name: String::from_str(env, "Discount Coupon"),
        points_required: 100,
        available_quantity: 5,
    });
    rewards.push_back(RedemptionOption {
        id: 3,
        name: String::from_str(env, "Free Shipping"),
        points_required: 50,
        available_quantity: 20,
    });
    rewards
}

pub fn create_premium_rewards(env: &Env) -> Vec<RedemptionOption> {
    let mut rewards = Vec::new(env);
    rewards.push_back(RedemptionOption {
        id: 1,
        name: String::from_str(env, "Premium Gift Card"),
        points_required: 1000,
        available_quantity: 2,
    });
    rewards.push_back(RedemptionOption {
        id: 2,
        name: String::from_str(env, "VIP Experience"),
        points_required: 5000,
        available_quantity: 1,
    });
    rewards
}

pub fn create_single_reward(env: &Env, id: u32, name: &str, points: u32, quantity: u32) -> Vec<RedemptionOption> {
    let mut rewards = Vec::new(env);
    rewards.push_back(RedemptionOption {
        id,
        name: String::from_str(env, name),
        points_required: points,
        available_quantity: quantity,
    });
    rewards
}

// ============ UTILITY FUNCTIONS ============

pub fn get_user_points(env: &Env, program_id: BytesN<32>, user_address: Address) -> u64 {
    let points_key = (
        soroban_sdk::Symbol::new(env, "points"),
        program_id,
        user_address,
    );
    env.storage()
        .persistent()
        .get::<(soroban_sdk::Symbol, BytesN<32>, Address), u64>(&points_key)
        .unwrap_or(0)
}

pub fn create_program_with_id(env: &Env, id: u8) -> BytesN<32> {
    let mut program_id_bytes = [0u8; 32];
    program_id_bytes[0] = id;
    BytesN::from_array(env, &program_id_bytes)
}

// ============ ASSERTION HELPERS ============

pub fn assert_reward_exists(rewards: &Vec<RedemptionOption>, reward_id: u32) {
    let exists = rewards.iter().any(|r| r.id == reward_id);
    assert!(exists, "Reward with ID {} should exist", reward_id);
}

pub fn assert_reward_not_exists(rewards: &Vec<RedemptionOption>, reward_id: u32) {
    let exists = rewards.iter().any(|r| r.id == reward_id);
    assert!(!exists, "Reward with ID {} should not exist", reward_id);
}

pub fn assert_reward_quantity(rewards: &Vec<RedemptionOption>, reward_id: u32, expected_quantity: u32) {
    let reward = rewards.iter().find(|r| r.id == reward_id).expect("Reward not found");
    assert_eq!(reward.available_quantity, expected_quantity,
               "Reward {} should have {} quantity", reward_id, expected_quantity);
}

// ============ CONTRACT WRAPPER FUNCTIONS ============

pub fn setup_loyalty_program(
    env: &Env,
    contract_address: &Address,
    program_id: BytesN<32>,
    points_per_transaction: u32,
    rewards: Vec<RedemptionOption>,
) {
    env.as_contract(contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id, points_per_transaction, rewards);
    });
}

pub fn setup_basic_program(env: &Env, contract_address: &Address, program_id: BytesN<32>) {
    let rewards = create_basic_rewards(env);
    setup_loyalty_program(env, contract_address, program_id, 1, rewards);
}

pub fn award_points_to_user(
    env: &Env,
    contract_address: &Address,
    program_id: BytesN<32>,
    user: Address,
    transaction_amount: u32,
) {
    env.as_contract(contract_address, || {
        LoyaltyContract::award_points(env.clone(), program_id, user, transaction_amount);
    });
}

pub fn redeem_reward_for_user(
    env: &Env,
    contract_address: &Address,
    program_id: BytesN<32>,
    user: Address,
    reward_id: u32,
) {
    env.as_contract(contract_address, || {
        LoyaltyContract::redeem_reward(env.clone(), program_id, user, reward_id);
    });
}

pub fn get_program_details(env: &Env, contract_address: &Address, program_id: BytesN<32>) -> LoyaltyProgram {
    env.as_contract(contract_address, || {
        LoyaltyContract::get_program_info(env.clone(), program_id)
    })
}

pub fn get_available_rewards(
    env: &Env,
    contract_address: &Address,
    program_id: BytesN<32>,
) -> Vec<RedemptionOption> {
    env.as_contract(contract_address, || {
        LoyaltyContract::list_available_rewards(env.clone(), program_id)
    })
}