#![cfg(test)]

use super::utils::*;

// ================================================================================
// INITIALIZATION TESTS
// ================================================================================

#[test]
fn test_initialize() {
    let ctx = setup_test();

    let result = ctx.client.initialize(&ctx.admin);
    assert_eq!(result, true);

    assert_eq!(ctx.client.get_admin(), ctx.admin);
    assert_eq!(ctx.client.get_farm_count(), 0);
}

// ================================================================================
// FARM CREATION TESTS
// ================================================================================

#[test]
fn test_create_farm_success() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let result = ctx.client.create_farm(
        &ctx.lp_token,
        &ctx.reward_token,
        &100_0000000,
        &150,
        &1100,
        &100000,
    );

    let farm_id = result;
    assert_eq!(farm_id, 0);
    assert_eq!(ctx.client.get_farm_count(), 1);

    let farm = ctx.client.get_farm(&farm_id);
    assert_eq!(farm.lp_token, ctx.lp_token);
    assert_eq!(farm.reward_token, ctx.reward_token);
    assert_eq!(farm.reward_per_block, 100_0000000);
    assert_eq!(farm.multiplier, 150);
    assert_eq!(farm.total_staked, 0);
    assert!(farm.is_active);
}

#[test]
fn test_create_multiple_farms() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm1 = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);
    let farm2 = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &200_0000000, &200, &1100, &100000);

    assert_eq!(farm1, 0);
    assert_eq!(farm2, 1);
    assert_eq!(ctx.client.get_farm_count(), 2);
}

// ================================================================================
// FARM MANAGEMENT TESTS
// ================================================================================

#[test]
fn test_update_farm() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    ctx.client.update_farm(&farm_id, &200_0000000, &200);

    let farm = ctx.client.get_farm(&farm_id);
    assert_eq!(farm.reward_per_block, 200_0000000);
    assert_eq!(farm.multiplier, 200);
}

#[test]
fn test_unpause_farm() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    ctx.client.set_farm_paused(&farm_id, &true);
    ctx.client.set_farm_paused(&farm_id, &false);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 1000_0000000);
    set_ledger_sequence(&ctx.env, 1200);

    assert_eq!(ctx.client.stake_lp(&ctx.farmer1, &farm_id, &1000_0000000), ());
}

#[test]
fn test_end_farm() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    set_ledger_sequence(&ctx.env, 5000);
    ctx.client.end_farm(&farm_id);

    let farm = ctx.client.get_farm(&farm_id);
    assert_eq!(farm.end_block, 5000);
    assert!(!farm.is_active);
}

#[test]
fn test_farm_with_same_lp_and_reward_token() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.lp_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.admin, 10_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.lp_token, &10_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 100);
    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

// ================================================================================
// ADMIN TESTS
// ================================================================================

#[test]
fn test_set_global_multiplier() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);
}

#[test]
fn test_deposit_rewards() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 1_000_000_0000000);

    let contract_balance_before = get_balance(&ctx.env, &ctx.reward_token, &ctx.client.address);
    ctx.client.deposit_rewards(&ctx.reward_token, &1_000_000_0000000);
    let contract_balance_after = get_balance(&ctx.env, &ctx.reward_token, &ctx.client.address);

    assert_eq!(contract_balance_after, contract_balance_before + 1_000_000_0000000);
}

#[test]
fn test_update_pool() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 100);

    ctx.client.update_pool(&farm_id);

    let farm = ctx.client.get_farm(&farm_id);
    assert_eq!(farm.last_reward_block, 1300);
}

#[test]
fn test_zero_total_staked() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    advance_ledger(&ctx.env, 100);

    ctx.client.update_pool(&farm_id);

    let farm = ctx.client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 0);
}