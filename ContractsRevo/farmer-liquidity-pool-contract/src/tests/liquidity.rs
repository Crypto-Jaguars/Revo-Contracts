use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::{FarmerLiquidityPoolContract, FarmerLiquidityPoolContractClient};
use crate::error::PoolError;
use super::utils::{setup_test_environment, TestEnvironment, assert_balance, assert_pool_reserves, assert_lp_balance, assert_approx_eq};

#[test]
fn test_add_liquidity_first_provider() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    let amount_a = 1000;
    let amount_b = 2000;

    let lp_tokens = test_env.add_liquidity(&test_env.user1, amount_a, amount_b);

    // For first liquidity provision, LP tokens should be sqrt(amount_a * amount_b)
    let expected_lp_tokens = (amount_a * amount_b).sqrt();
    assert_eq!(lp_tokens, expected_lp_tokens);

    // Check reserves
    assert_pool_reserves(&test_env, amount_a, amount_b);

    // Check LP token balance
    assert_lp_balance(&test_env, &test_env.user1, lp_tokens);

    // Check token balances
    assert_balance(&env, &test_env.token_a, &test_env.user1, 99_000); // 100_000 - 1_000
    assert_balance(&env, &test_env.token_b, &test_env.user1, 98_000); // 100_000 - 2_000
}

#[test]
fn test_add_liquidity_second_provider() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // First provider adds liquidity
    let lp_tokens_1 = test_env.add_liquidity(&test_env.user1, 1000, 2000);

    // Second provider adds liquidity with same ratio
    let lp_tokens_2 = test_env.add_liquidity(&test_env.user2, 500, 1000);

    // Second provider should get proportional LP tokens
    let expected_lp_tokens_2 = (lp_tokens_1 * 500) / 1000; // Proportional to amount_a
    assert_approx_eq(lp_tokens_2, expected_lp_tokens_2, 1);

    // Check total reserves
    assert_pool_reserves(&test_env, 1500, 3000);

    // Check LP token balances
    assert_lp_balance(&test_env, &test_env.user1, lp_tokens_1);
    assert_lp_balance(&test_env, &test_env.user2, lp_tokens_2);
}

#[test]
fn test_add_liquidity_mismatched_ratio() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // First provider adds liquidity
    test_env.add_liquidity(&test_env.user1, 1000, 2000);

    // Second provider adds liquidity with different ratio
    // Should get LP tokens based on the smaller ratio
    let lp_tokens_2 = test_env.add_liquidity(&test_env.user2, 1000, 1000); // Different ratio

    // Should get LP tokens based on the smaller amount (1000/1000 vs 1000/2000)
    let expected_lp_tokens_2 = (1000 * 2000) / 1000; // Based on token_a ratio
    assert_approx_eq(lp_tokens_2, expected_lp_tokens_2, 1);
}

#[test]
fn test_add_liquidity_zero_amounts() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Try to add zero liquidity
    let result = std::panic::catch_unwind(|| {
        test_env.add_liquidity(&test_env.user1, 0, 1000);
    });
    assert!(result.is_err());

    let result = std::panic::catch_unwind(|| {
        test_env.add_liquidity(&test_env.user1, 1000, 0);
    });
    assert!(result.is_err());
}

#[test]
fn test_add_liquidity_insufficient_balance() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Try to add more liquidity than user has
    let result = std::panic::catch_unwind(|| {
        test_env.add_liquidity(&test_env.user1, 200_000, 1000); // More than user has
    });
    assert!(result.is_err());
}

#[test]
fn test_remove_liquidity() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add liquidity
    let lp_tokens = test_env.add_liquidity(&test_env.user1, 1000, 2000);
    let initial_balance_a = 99_000;
    let initial_balance_b = 98_000;

    // Remove half of the liquidity
    let (amount_a, amount_b) = test_env.remove_liquidity(&test_env.user1, lp_tokens / 2);

    // Should get proportional amounts back
    assert_approx_eq(amount_a, 500, 1);
    assert_approx_eq(amount_b, 1000, 1);

    // Check reserves
    assert_pool_reserves(&test_env, 500, 1000);

    // Check LP token balance
    assert_lp_balance(&test_env, &test_env.user1, lp_tokens / 2);

    // Check token balances
    assert_balance(&env, &test_env.token_a, &test_env.user1, initial_balance_a + amount_a);
    assert_balance(&env, &test_env.token_b, &test_env.user1, initial_balance_b + amount_b);
}

#[test]
fn test_remove_all_liquidity() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add liquidity
    let lp_tokens = test_env.add_liquidity(&test_env.user1, 1000, 2000);

    // Remove all liquidity
    let (amount_a, amount_b) = test_env.remove_liquidity(&test_env.user1, lp_tokens);

    // Should get all amounts back
    assert_eq!(amount_a, 1000);
    assert_eq!(amount_b, 2000);

    // Check reserves are zero
    assert_pool_reserves(&test_env, 0, 0);

    // Check LP token balance is zero
    assert_lp_balance(&test_env, &test_env.user1, 0);
}

#[test]
fn test_remove_liquidity_insufficient_balance() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add liquidity
    test_env.add_liquidity(&test_env.user1, 1000, 2000);

    // Try to remove more LP tokens than user has
    let result = std::panic::catch_unwind(|| {
        test_env.remove_liquidity(&test_env.user1, 1_000_000);
    });
    assert!(result.is_err());
}

#[test]
fn test_remove_liquidity_zero_lp_tokens() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add liquidity
    test_env.add_liquidity(&test_env.user1, 1000, 2000);

    // Try to remove zero LP tokens
    let result = std::panic::catch_unwind(|| {
        test_env.remove_liquidity(&test_env.user1, 0);
    });
    assert!(result.is_err());
}

#[test]
fn test_remove_liquidity_from_empty_pool() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Try to remove liquidity from empty pool
    let result = std::panic::catch_unwind(|| {
        test_env.remove_liquidity(&test_env.user1, 1000);
    });
    assert!(result.is_err());
}

#[test]
fn test_multiple_liquidity_providers() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Three providers add liquidity
    let lp_tokens_1 = test_env.add_liquidity(&test_env.user1, 1000, 2000);
    let lp_tokens_2 = test_env.add_liquidity(&test_env.user2, 500, 1000);
    let lp_tokens_3 = test_env.add_liquidity(&test_env.user3, 2000, 4000);

    // Check total reserves
    assert_pool_reserves(&test_env, 3500, 7000);

    // Check LP token balances
    assert_lp_balance(&test_env, &test_env.user1, lp_tokens_1);
    assert_lp_balance(&test_env, &test_env.user2, lp_tokens_2);
    assert_lp_balance(&test_env, &test_env.user3, lp_tokens_3);

    // User2 removes all their liquidity
    let (amount_a, amount_b) = test_env.remove_liquidity(&test_env.user2, lp_tokens_2);
    assert_approx_eq(amount_a, 500, 1);
    assert_approx_eq(amount_b, 1000, 1);

    // Check reserves after removal
    assert_pool_reserves(&test_env, 3000, 6000);

    // Check LP token balance is zero
    assert_lp_balance(&test_env, &test_env.user2, 0);
}

#[test]
fn test_liquidity_provision_events() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add liquidity
    let lp_tokens = test_env.add_liquidity(&test_env.user1, 1000, 2000);

    // Check events
    let events = env.events().all();
    assert!(events.len() >= 2); // Initialization + liquidity added

    // Find liquidity added event
    let liquidity_event = events.iter().find(|e| e.0 == soroban_sdk::symbol_short!("liquidity_added"));
    assert!(liquidity_event.is_some());
}

#[test]
fn test_liquidity_removal_events() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add and remove liquidity
    let lp_tokens = test_env.add_liquidity(&test_env.user1, 1000, 2000);
    test_env.remove_liquidity(&test_env.user1, lp_tokens);

    // Check events
    let events = env.events().all();
    assert!(events.len() >= 3); // Initialization + liquidity added + liquidity removed

    // Find liquidity removed event
    let liquidity_event = events.iter().find(|e| e.0 == soroban_sdk::symbol_short!("liquidity_removed"));
    assert!(liquidity_event.is_some());
}

#[test]
fn test_lp_token_precision() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add liquidity with amounts that might cause precision issues
    let lp_tokens = test_env.add_liquidity(&test_env.user1, 1, 1);

    // Should still work with small amounts
    assert!(lp_tokens > 0);
    assert_lp_balance(&test_env, &test_env.user1, lp_tokens);
}

#[test]
fn test_liquidity_provision_before_initialization() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Try to add liquidity before initialization
    let result = std::panic::catch_unwind(|| {
        test_env.add_liquidity(&test_env.user1, 1000, 2000);
    });
    assert!(result.is_err());
}
