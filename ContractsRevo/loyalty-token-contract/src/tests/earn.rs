#![cfg(test)]

use super::utils::*;
use crate::LoyaltyContract;

#[test]
fn test_award_points_after_transaction() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_rewards(&env);
    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 50);
        // Points per transaction = 1, amount = 50, expect 50 points

        let points: u64 = get_user_points(&env, program_id, user.clone());
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
        let points: u64 = get_user_points(&env, program_id, user.clone());

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
        let points: u64 = get_user_points(&env, program_id, user.clone());

        assert_eq!(points, 150);
    });
}

// ============ NEW COMPREHENSIVE TESTS ============

#[test]
fn test_basic_point_awarding() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_basic_rewards(&env);

    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 50);

        let points = get_user_points(&env, program_id, user);
        assert_eq!(points, 50);
    });
}

#[test]
fn test_dynamic_point_rates() {
    let (env, contract_address, _program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_basic_rewards(&env);

    // Test different point rates
    let test_cases = [(2, 100, 200), (5, 50, 250), (10, 20, 200)];

    for (i, (rate, amount, expected)) in test_cases.iter().enumerate() {
        let test_program_id = create_program_with_id(&env, i as u8 + 2);

        env.as_contract(&contract_address, || {
            LoyaltyContract::create_loyalty_program(
                env.clone(),
                test_program_id.clone(),
                *rate,
                rewards.clone(),
            );
            LoyaltyContract::award_points(
                env.clone(),
                test_program_id.clone(),
                user.clone(),
                *amount,
            );

            let points = get_user_points(&env, test_program_id, user.clone());
            assert_eq!(points, *expected);
        });
    }
}

#[test]
fn test_cumulative_point_accumulation() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_basic_rewards(&env);

    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);

        // Multiple transactions
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 50);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 30);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 20);

        let points = get_user_points(&env, program_id, user);
        assert_eq!(points, 100);
    });
}
