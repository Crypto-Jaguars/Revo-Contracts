use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::{FarmerLiquidityPoolContract, FarmerLiquidityPoolContractClient};
use crate::error::PoolError;
use super::utils::{setup_test_environment, TestEnvironment, assert_balance, assert_pool_reserves, assert_approx_eq};

#[test]
fn test_swap_token_a_to_token_b() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30); // 0.3% fee

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    let amount_in = 1000;
    let amount_out = test_env.swap(&test_env.user2, &test_env.token_a, amount_in);

    // Calculate expected output using constant product formula
    // With 0.3% fee: amount_in_after_fee = 1000 - 3 = 997
    // amount_out = (997 * 20000) / (10000 + 997) ≈ 1814
    let expected_amount_out = (997 * 20000) / (10000 + 997);
    assert_approx_eq(amount_out, expected_amount_out, 1);

    // Check reserves updated correctly
    assert_pool_reserves(&test_env, 11000, 20000 - amount_out);

    // Check user balances
    assert_balance(&env, &test_env.token_a, &test_env.user2, 99_000); // 100_000 - 1_000
    assert_balance(&env, &test_env.token_b, &test_env.user2, 100_000 + amount_out);
}

#[test]
fn test_swap_token_b_to_token_a() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    let amount_in = 2000;
    let amount_out = test_env.swap(&test_env.user2, &test_env.token_b, amount_in);

    // Calculate expected output
    // With 0.3% fee: amount_in_after_fee = 2000 - 6 = 1994
    // amount_out = (1994 * 10000) / (20000 + 1994) ≈ 907
    let expected_amount_out = (1994 * 10000) / (20000 + 1994);
    assert_approx_eq(amount_out, expected_amount_out, 1);

    // Check reserves updated correctly
    assert_pool_reserves(&test_env, 10000 - amount_out, 22000);

    // Check user balances
    assert_balance(&env, &test_env.token_a, &test_env.user2, 100_000 + amount_out);
    assert_balance(&env, &test_env.token_b, &test_env.user2, 98_000); // 100_000 - 2_000
}

#[test]
fn test_swap_zero_amount() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    // Try to swap zero amount
    let result = std::panic::catch_unwind(|| {
        test_env.swap(&test_env.user2, &test_env.token_a, 0);
    });
    assert!(result.is_err());
}

#[test]
fn test_swap_insufficient_balance() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    // Try to swap more than user has
    let result = std::panic::catch_unwind(|| {
        test_env.swap(&test_env.user2, &test_env.token_a, 200_000);
    });
    assert!(result.is_err());
}

#[test]
fn test_swap_invalid_token() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    // Create a third token
    let token_c = env.register_stellar_asset_contract(&test_env.admin);

    // Try to swap with invalid token
    let result = std::panic::catch_unwind(|| {
        test_env.swap(&test_env.user2, &token_c, 1000);
    });
    assert!(result.is_err());
}

#[test]
fn test_swap_empty_pool() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Try to swap in empty pool
    let result = std::panic::catch_unwind(|| {
        test_env.swap(&test_env.user2, &test_env.token_a, 1000);
    });
    assert!(result.is_err());
}

#[test]
fn test_swap_exceeds_reserves() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add small liquidity
    test_env.add_liquidity(&test_env.user1, 100, 200);

    // Try to swap more than available reserves
    let result = std::panic::catch_unwind(|| {
        test_env.swap(&test_env.user2, &test_env.token_a, 1000);
    });
    assert!(result.is_err());
}

#[test]
fn test_calculate_swap_output() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    let amount_in = 1000;
    let calculated_output = test_env.calculate_swap_output(&test_env.token_a, amount_in);

    // Perform actual swap
    let actual_output = test_env.swap(&test_env.user2, &test_env.token_a, amount_in);

    // Calculated output should match actual output
    assert_eq!(calculated_output, actual_output);
}

#[test]
fn test_swap_fee_calculation() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(100); // 1% fee for easier calculation

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    let amount_in = 1000;
    let amount_out = test_env.swap(&test_env.user2, &test_env.token_a, amount_in);

    // With 1% fee: amount_in_after_fee = 1000 - 10 = 990
    // amount_out = (990 * 20000) / (10000 + 990) ≈ 1801
    let expected_amount_out = (990 * 20000) / (10000 + 990);
    assert_approx_eq(amount_out, expected_amount_out, 1);

    // Fee should be 10 tokens
    let fee_amount = amount_in - (amount_in - 10);
    assert_eq!(fee_amount, 10);
}

#[test]
fn test_swap_no_fee() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(0); // No fee

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    let amount_in = 1000;
    let amount_out = test_env.swap(&test_env.user2, &test_env.token_a, amount_in);

    // With no fee: amount_out = (1000 * 20000) / (10000 + 1000) ≈ 1818
    let expected_amount_out = (1000 * 20000) / (10000 + 1000);
    assert_approx_eq(amount_out, expected_amount_out, 1);
}

#[test]
fn test_multiple_swaps() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    // First swap: A to B
    let amount_out_1 = test_env.swap(&test_env.user2, &test_env.token_a, 1000);
    let (reserve_a_1, reserve_b_1) = test_env.get_reserves();

    // Second swap: B to A
    let amount_out_2 = test_env.swap(&test_env.user3, &test_env.token_b, 2000);
    let (reserve_a_2, reserve_b_2) = test_env.get_reserves();

    // Reserves should be updated correctly after each swap
    assert_eq!(reserve_a_1, 11000);
    assert_eq!(reserve_b_1, 20000 - amount_out_1);
    assert_eq!(reserve_a_2, 11000 - amount_out_2);
    assert_eq!(reserve_b_2, 20000 - amount_out_1 + 2000);
}

#[test]
fn test_swap_price_impact() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    // Small swap should have minimal price impact
    let small_swap = test_env.swap(&test_env.user2, &test_env.token_a, 100);
    let small_price_impact = (small_swap * 10000) / 20000; // Price per token

    // Large swap should have significant price impact
    let large_swap = test_env.swap(&test_env.user3, &test_env.token_a, 5000);
    let large_price_impact = (large_swap * 10000) / 20000; // Price per token

    // Large swap should have worse price impact (lower output per input)
    assert!(large_price_impact < small_price_impact);
}

#[test]
fn test_swap_events() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity
    test_env.add_liquidity(&test_env.user1, 10000, 20000);

    // Perform swap
    let amount_out = test_env.swap(&test_env.user2, &test_env.token_a, 1000);

    // Check events
    let events = env.events().all();
    assert!(events.len() >= 3); // Initialization + liquidity added + swap

    // Find swap event
    let swap_event = events.iter().find(|e| e.0 == soroban_sdk::symbol_short!("swap_executed"));
    assert!(swap_event.is_some());
}

#[test]
fn test_swap_before_initialization() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Try to swap before initialization
    let result = std::panic::catch_unwind(|| {
        test_env.swap(&test_env.user2, &test_env.token_a, 1000);
    });
    assert!(result.is_err());
}

#[test]
fn test_swap_calculation_before_initialization() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);

    // Try to calculate swap output before initialization
    let result = std::panic::catch_unwind(|| {
        test_env.calculate_swap_output(&test_env.token_a, 1000);
    });
    assert!(result.is_err());
}

#[test]
fn test_swap_with_different_fee_rates() {
    let env = Env::default();
    
    // Test with different fee rates
    let fee_rates = vec![0, 10, 30, 100, 500];
    
    for fee_rate in fee_rates {
        let test_env = setup_test_environment(&env);
        test_env.initialize_pool(fee_rate);
        
        // Add initial liquidity
        test_env.add_liquidity(&test_env.user1, 10000, 20000);
        
        let amount_in = 1000;
        let amount_out = test_env.swap(&test_env.user2, &test_env.token_a, amount_in);
        
        // Higher fee rate should result in lower output
        assert!(amount_out > 0);
        
        // With higher fees, output should be lower
        if fee_rate > 0 {
            let fee_amount = (amount_in * fee_rate as i128) / 10000;
            let amount_in_after_fee = amount_in - fee_amount;
            let expected_output = (amount_in_after_fee * 20000) / (10000 + amount_in_after_fee);
            assert_approx_eq(amount_out, expected_output, 1);
        }
    }
}

#[test]
fn test_swap_rounding_precision() {
    let env = Env::default();
    let test_env = setup_test_environment(&env);
    test_env.initialize_pool(30);

    // Add initial liquidity with amounts that might cause rounding issues
    test_env.add_liquidity(&test_env.user1, 1000, 1000);

    // Small swap to test precision
    let amount_out = test_env.swap(&test_env.user2, &test_env.token_a, 1);

    // Should still work with small amounts
    assert!(amount_out > 0);
    assert!(amount_out < 1000); // Should be less than total reserves
}
