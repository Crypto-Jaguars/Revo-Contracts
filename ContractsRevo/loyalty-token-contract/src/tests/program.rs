#![cfg(test)]

use super::utils::*;
use crate::LoyaltyContract;
use soroban_sdk::{String, Vec};

#[test]
fn test_create_basic_loyalty_program() {
    let (env, contract_address, program_id) = setup_test();
    let rewards = create_basic_rewards(&env);

    setup_loyalty_program(
        &env,
        &contract_address,
        program_id.clone(),
        1,
        rewards.clone(),
    );

    let program = get_program_details(&env, &contract_address, program_id.clone());
    assert_eq!(program.program_id, program_id);
    assert_eq!(program.points_per_transaction, 1);
    assert_eq!(program.redemption_options.len(), 3);
}

#[test]
fn test_create_program_with_different_point_rates() {
    let (env, contract_address, _default_program_id) = setup_test();
    let rewards = create_basic_rewards(&env);

    let test_rates = [1, 5, 10, 100, 1000];

    for (i, &rate) in test_rates.iter().enumerate() {
        let program_id = create_program_with_id(&env, i as u8 + 2);
        setup_loyalty_program(
            &env,
            &contract_address,
            program_id.clone(),
            rate,
            rewards.clone(),
        );

        let program = get_program_details(&env, &contract_address, program_id);
        assert_eq!(program.points_per_transaction, rate);
    }
}

#[test]
fn test_create_program_with_no_rewards() {
    let (env, contract_address, program_id) = setup_test();
    let empty_rewards = Vec::new(&env);

    setup_loyalty_program(
        &env,
        &contract_address,
        program_id.clone(),
        1,
        empty_rewards,
    );

    let program = get_program_details(&env, &contract_address, program_id);
    assert_eq!(program.redemption_options.len(), 0);
}

#[test]
fn test_create_program_with_single_reward() {
    let (env, contract_address, program_id) = setup_test();
    let single_reward = create_single_reward(&env, 1, "Exclusive Gift", 500, 1);

    setup_loyalty_program(
        &env,
        &contract_address,
        program_id.clone(),
        2,
        single_reward,
    );

    let program = get_program_details(&env, &contract_address, program_id);
    assert_eq!(program.redemption_options.len(), 1);

    let reward = program.redemption_options.get(0).unwrap();
    assert_eq!(reward.id, 1);
    assert_eq!(reward.name, String::from_str(&env, "Exclusive Gift"));
    assert_eq!(reward.points_required, 500);
    assert_eq!(reward.available_quantity, 1);
}

#[test]
#[should_panic(expected = "Program already exists")]
fn test_create_duplicate_program() {
    let (env, contract_address, program_id) = setup_test();
    let rewards = create_basic_rewards(&env);

    // Create program first time
    setup_loyalty_program(
        &env,
        &contract_address,
        program_id.clone(),
        1,
        rewards.clone(),
    );

    // Attempt to create same program again
    setup_loyalty_program(&env, &contract_address, program_id, 2, rewards);
}

#[test]
fn test_create_multiple_different_programs() {
    let (env, contract_address, _default_program_id) = setup_test();
    let basic_rewards = create_basic_rewards(&env);
    let premium_rewards = create_premium_rewards(&env);

    let program1 = create_program_with_id(&env, 10);
    let program2 = create_program_with_id(&env, 11);
    let program3 = create_program_with_id(&env, 12);

    setup_loyalty_program(&env, &contract_address, program1.clone(), 1, basic_rewards);
    setup_loyalty_program(
        &env,
        &contract_address,
        program2.clone(),
        5,
        premium_rewards,
    );
    setup_loyalty_program(
        &env,
        &contract_address,
        program3.clone(),
        10,
        Vec::new(&env),
    );

    // Verify each program
    let retrieved_program1 = get_program_details(&env, &contract_address, program1);
    let retrieved_program2 = get_program_details(&env, &contract_address, program2);
    let retrieved_program3 = get_program_details(&env, &contract_address, program3);

    assert_eq!(retrieved_program1.points_per_transaction, 1);
    assert_eq!(retrieved_program1.redemption_options.len(), 3);

    assert_eq!(retrieved_program2.points_per_transaction, 5);
    assert_eq!(retrieved_program2.redemption_options.len(), 2);

    assert_eq!(retrieved_program3.points_per_transaction, 10);
    assert_eq!(retrieved_program3.redemption_options.len(), 0);
}

#[test]
#[should_panic(expected = "Program not found")]
fn test_get_nonexistent_program() {
    let (env, contract_address, _program_id) = setup_test();
    let fake_program_id = create_program_with_id(&env, 99);

    get_program_details(&env, &contract_address, fake_program_id);
}

#[test]
fn test_program_with_varying_reward_quantities() {
    let (env, contract_address, program_id) = setup_test();

    let mut rewards = Vec::new(&env);
    rewards.push_back(create_test_reward(&env, 1, "Common", 50, 100));
    rewards.push_back(create_test_reward(&env, 2, "Rare", 200, 10));
    rewards.push_back(create_test_reward(&env, 3, "Ultra Rare", 1000, 1));

    setup_loyalty_program(&env, &contract_address, program_id.clone(), 1, rewards);

    let program = get_program_details(&env, &contract_address, program_id);
    assert_eq!(program.redemption_options.len(), 3);

    let common = program
        .redemption_options
        .iter()
        .find(|r| r.id == 1)
        .unwrap();
    let rare = program
        .redemption_options
        .iter()
        .find(|r| r.id == 2)
        .unwrap();
    let ultra_rare = program
        .redemption_options
        .iter()
        .find(|r| r.id == 3)
        .unwrap();

    assert_eq!(common.available_quantity, 100);
    assert_eq!(rare.available_quantity, 10);
    assert_eq!(ultra_rare.available_quantity, 1);
}

#[test]
fn test_program_with_varying_point_requirements() {
    let (env, contract_address, program_id) = setup_test();

    let mut rewards = Vec::new(&env);
    rewards.push_back(create_test_reward(&env, 1, "Low Cost", 10, 50));
    rewards.push_back(create_test_reward(&env, 2, "Medium Cost", 250, 20));
    rewards.push_back(create_test_reward(&env, 3, "High Cost", 5000, 5));

    setup_loyalty_program(&env, &contract_address, program_id.clone(), 1, rewards);

    let program = get_program_details(&env, &contract_address, program_id);

    let low_cost = program
        .redemption_options
        .iter()
        .find(|r| r.id == 1)
        .unwrap();
    let medium_cost = program
        .redemption_options
        .iter()
        .find(|r| r.id == 2)
        .unwrap();
    let high_cost = program
        .redemption_options
        .iter()
        .find(|r| r.id == 3)
        .unwrap();

    assert_eq!(low_cost.points_required, 10);
    assert_eq!(medium_cost.points_required, 250);
    assert_eq!(high_cost.points_required, 5000);
}

#[test]
fn test_program_persistence() {
    let (env, contract_address, program_id) = setup_test();
    let rewards = create_basic_rewards(&env);

    setup_loyalty_program(&env, &contract_address, program_id.clone(), 3, rewards);

    // Retrieve program multiple times to test persistence
    for _ in 0..5 {
        let program = get_program_details(&env, &contract_address, program_id.clone());
        assert_eq!(program.program_id, program_id);
        assert_eq!(program.points_per_transaction, 3);
        assert_eq!(program.redemption_options.len(), 3);
    }
}

#[test]
fn test_program_with_zero_point_rate() {
    let (env, contract_address, program_id) = setup_test();
    let rewards = create_basic_rewards(&env);

    setup_loyalty_program(&env, &contract_address, program_id.clone(), 0, rewards);

    let program = get_program_details(&env, &contract_address, program_id);
    assert_eq!(program.points_per_transaction, 0);
}

#[test]
fn test_program_with_max_point_rate() {
    let (env, contract_address, program_id) = setup_test();
    let rewards = create_basic_rewards(&env);

    setup_loyalty_program(
        &env,
        &contract_address,
        program_id.clone(),
        u32::MAX,
        rewards,
    );

    let program = get_program_details(&env, &contract_address, program_id);
    assert_eq!(program.points_per_transaction, u32::MAX);
}

#[test]
fn test_program_reward_modifications_after_creation() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);

    setup_basic_program(&env, &contract_address, program_id.clone());

    // Award points and redeem to modify inventory
    award_points_to_user(
        &env,
        &contract_address,
        program_id.clone(),
        user.clone(),
        200,
    );
    redeem_reward_for_user(&env, &contract_address, program_id.clone(), user, 1);

    // Check that program reflects inventory changes
    let program = get_program_details(&env, &contract_address, program_id);
    let gift_card = program
        .redemption_options
        .iter()
        .find(|r| r.id == 1)
        .unwrap();
    assert_eq!(gift_card.available_quantity, 9); // Reduced from 10 to 9
}

#[test]
fn test_list_available_rewards() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);

    setup_basic_program(&env, &contract_address, program_id.clone());

    // Initially all rewards should be available
    let available_rewards = get_available_rewards(&env, &contract_address, program_id.clone());
    assert_eq!(available_rewards.len(), 3);

    // Exhaust one type of reward
    award_points_to_user(
        &env,
        &contract_address,
        program_id.clone(),
        user.clone(),
        500,
    );

    // Redeem all Discount Coupons (5 available, 100 points each)
    for _ in 0..5 {
        redeem_reward_for_user(&env, &contract_address, program_id.clone(), user.clone(), 2);
    }

    // Now only 2 rewards should be available
    let available_rewards = get_available_rewards(&env, &contract_address, program_id);
    assert_eq!(available_rewards.len(), 2);

    // Verify the out-of-stock reward is not in the list
    assert_reward_not_exists(&available_rewards, 2);
    assert_reward_exists(&available_rewards, 1);
    assert_reward_exists(&available_rewards, 3);
}

#[test]
fn test_program_with_large_number_of_rewards() {
    let (env, contract_address, program_id) = setup_test();

    let mut rewards = Vec::new(&env);
    for i in 1..=50 {
        let reward_name = if i <= 10 {
            "Reward Low"
        } else if i <= 30 {
            "Reward Med"
        } else {
            "Reward High"
        };
        rewards.push_back(create_test_reward(&env, i, reward_name, i * 10, i));
    }

    setup_loyalty_program(&env, &contract_address, program_id.clone(), 1, rewards);

    let program = get_program_details(&env, &contract_address, program_id);
    assert_eq!(program.redemption_options.len(), 50);

    // Verify first and last rewards
    let first_reward = program
        .redemption_options
        .iter()
        .find(|r| r.id == 1)
        .unwrap();
    let last_reward = program
        .redemption_options
        .iter()
        .find(|r| r.id == 50)
        .unwrap();

    assert_eq!(first_reward.points_required, 10);
    assert_eq!(first_reward.available_quantity, 1);
    assert_eq!(last_reward.points_required, 500);
    assert_eq!(last_reward.available_quantity, 50);
}

// ============ CROSS-PROGRAM TESTS ============

#[test]
#[should_panic(expected = "Insufficient points")]
fn test_cross_program_insufficient_points() {
    let (env, contract_address, _default_program_id) = setup_test();
    let user = create_user(&env);

    let program1 = create_program_with_id(&env, 10);
    let program2 = create_program_with_id(&env, 11);

    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(
            env.clone(),
            program1,
            1,
            create_basic_rewards(&env),
        );
        LoyaltyContract::create_loyalty_program(
            env.clone(),
            program2.clone(),
            3,
            create_premium_rewards(&env),
        );

        LoyaltyContract::award_points(env.clone(), program2.clone(), user.clone(), 200);

        // Should fail - user has 600 points in program2 but premium reward needs 1000
        LoyaltyContract::redeem_reward(env.clone(), program2, user, 1);
    });
}

// Helper function for creating test rewards
fn create_test_reward(
    env: &soroban_sdk::Env,
    id: u32,
    name: &str,
    points: u32,
    quantity: u32,
) -> crate::RedemptionOption {
    crate::RedemptionOption {
        id,
        name: soroban_sdk::String::from_str(env, name),
        points_required: points,
        available_quantity: quantity,
    }
}
