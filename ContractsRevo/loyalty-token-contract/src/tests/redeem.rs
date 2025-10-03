#![cfg(test)]

use super::utils::*;
use crate::LoyaltyContract;

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
        let points: u64 = get_user_points(&env, program_id.clone(), user.clone());

        assert_eq!(points, 0);
        // Check inventory
        let program = LoyaltyContract::get_program_info(env.clone(), program_id);
        let gift_card = program
            .redemption_options
            .iter()
            .find(|opt| opt.id == 1)
            .unwrap();
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
    let user1 = create_user(&env);
    let user2 = create_user(&env);
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

// ============ NEW COMPREHENSIVE TESTS ============

#[test]
fn test_successful_reward_redemption() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_basic_rewards(&env);

    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);

        // Award enough points for redemption
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 200);

        // Redeem Gift Card (id=1, requires 200 points)
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 1);

        // Check points are deducted
        let remaining_points = get_user_points(&env, program_id.clone(), user);
        assert_eq!(remaining_points, 0);

        // Check inventory is updated
        let program = LoyaltyContract::get_program_info(env.clone(), program_id);
        let gift_card = program
            .redemption_options
            .iter()
            .find(|opt| opt.id == 1)
            .unwrap();
        assert_eq!(gift_card.available_quantity, 9); // Started with 10, now 9
    });
}

#[test]
fn test_partial_point_redemption() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_basic_rewards(&env);

    env.as_contract(&contract_address, || {
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);

        // Award more points than needed for redemption
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 300);

        // Redeem Discount Coupon (id=2, requires 100 points)
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 2);

        // Check remaining points
        let remaining_points = get_user_points(&env, program_id, user);
        assert_eq!(remaining_points, 200);
    });
}

// ============ END-TO-END INTEGRATION TEST ============

#[test]
fn test_full_loyalty_program_lifecycle() {
    let (env, contract_address, program_id) = setup_test();
    let user = create_user(&env);
    let rewards = create_basic_rewards(&env);

    env.as_contract(&contract_address, || {
        // 1. Create loyalty program
        LoyaltyContract::create_loyalty_program(env.clone(), program_id.clone(), 1, rewards);

        // 2. Verify program creation
        let program = LoyaltyContract::get_program_info(env.clone(), program_id.clone());
        assert_eq!(program.redemption_options.len(), 3);

        // 3. User makes purchases and earns points
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 100);
        LoyaltyContract::award_points(env.clone(), program_id.clone(), user.clone(), 150);

        let total_points = get_user_points(&env, program_id.clone(), user.clone());
        assert_eq!(total_points, 250);

        // 4. User redeems rewards
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 2); // 100 points
        LoyaltyContract::redeem_reward(env.clone(), program_id.clone(), user.clone(), 3); // 50 points

        // 5. Verify final state
        let remaining_points = get_user_points(&env, program_id.clone(), user.clone());
        assert_eq!(remaining_points, 100);

        let updated_program = LoyaltyContract::get_program_info(env.clone(), program_id);
        let discount_coupon = updated_program
            .redemption_options
            .iter()
            .find(|r| r.id == 2)
            .unwrap();
        let free_shipping = updated_program
            .redemption_options
            .iter()
            .find(|r| r.id == 3)
            .unwrap();

        assert_eq!(discount_coupon.available_quantity, 4);
        assert_eq!(free_shipping.available_quantity, 19);
    });
}
