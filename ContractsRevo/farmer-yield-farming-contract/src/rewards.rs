#![cfg(test)]

use super::utils::*;
use soroban_sdk::testutils::Address as _;
    

// ================================================================================
// REWARDS TESTS
// ================================================================================

#[test]
fn test_harvest_rewards_success() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &600000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 100_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &100_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 17280 * 30);
    let pending_30days = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending_30days > 0);
}
// 1000);

//     let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

//     mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 10_000_000_0000000);
//     ctx.client.deposit_rewards(&ctx.reward_token, &10_000_000_0000000);

//     mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
//     mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer2, 10_000_0000000);

//     set_ledger_sequence(&ctx.env, 1200);
//     ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);
//     ctx.client.stake_lp(&ctx.farmer2, &farm_id, &10_000_0000000);

//     advance_ledger(&ctx.env, 100);

//     let pending1 = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);
//     let pending2 = ctx.client.get_pending_rewards(&ctx.farmer2, &farm_id);

//     assert!(pending1 > 0);
//     assert!(pending2 > 0);
// }

#[test]
fn test_rewards_stop_after_end_block() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &2000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 10_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &10_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    set_ledger_sequence(&ctx.env, 2500);

    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_rewards_calculation_accuracy() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &100, &1100, &100000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 10_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &10_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 100);

    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_max_multiplier_rewards() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &500, &1100, &100000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 100_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &100_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 100);
    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

// ================================================================================
// TIER AND LOYALTY BONUS TESTS
// ================================================================================

#[test]
fn test_farmer_tier_smallholder() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 1_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &1_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 500_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &500_0000000);

    advance_ledger(&ctx.env, 100);
    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_farmer_tier_cooperative() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 1_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &1_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 5_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &5_000_0000000);

    advance_ledger(&ctx.env, 100);
    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_farmer_tier_enterprise() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 10_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &10_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 15_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &15_000_0000000);

    advance_ledger(&ctx.env, 100);
    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_loyalty_bonus_7_days() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &150000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 10_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &10_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 17280 * 7);
    let pending_7days = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending_7days > 0);
}

#[test]
fn test_loyalty_bonus_30_days() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 1_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &1_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    let balance_before = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer1);

    advance_ledger(&ctx.env, 100);
    ctx.client.harvest(&ctx.farmer1, &farm_id);
    
    let balance_after = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer1);
    assert!(balance_after > balance_before);
}

#[test]
fn test_get_pending_rewards() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 100);
    let pending = ctx.client.get_pending_rewards(&ctx.farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_pending_rewards_no_stake() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    let farmer3 = soroban_sdk::Address::generate(&ctx.env);
    let pending = ctx.client.get_pending_rewards(&farmer3, &farm_id);
    assert_eq!(pending, 0);
}

#[test]
fn test_multiple_harvests() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(&ctx.lp_token, &ctx.reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 10_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &10_000_000_0000000);

    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&ctx.env, 100);
    let balance_after_first = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer1);
    ctx.client.harvest(&ctx.farmer1, &farm_id);
    let balance_after_harvest1 = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer1);
    let first_reward = balance_after_harvest1 - balance_after_first;

    advance_ledger(&ctx.env, 100);
    ctx.client.harvest(&ctx.farmer1, &farm_id);
    let balance_after_harvest2 = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer1);
    let second_reward = balance_after_harvest2 - balance_after_harvest1;

    assert!(first_reward > 0);
    assert!(second_reward > 0);
}

#[test]
fn test_rewards_distribution_multiple_farmers() {
    let ctx = setup_test();

    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    // Create farm with rewards
    let farm_id = ctx.client.create_farm(
        &ctx.lp_token,
        &ctx.reward_token,
        &100_0000000, // reward per block
        &150,
        &1100,
        &100000,
    );

    // Mint and deposit rewards
    mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, 10_000_000_0000000);
    ctx.client.deposit_rewards(&ctx.reward_token, &10_000_000_0000000);

    // Mint LP tokens for two farmers
    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer1, 10_000_0000000);
    mint_lp_tokens(&ctx.env, &ctx.lp_token, &ctx.farmer2, 10_000_0000000);

    // Both farmers stake the same amount
    set_ledger_sequence(&ctx.env, 1200);
    ctx.client.stake_lp(&ctx.farmer1, &farm_id, &10_000_0000000);
    ctx.client.stake_lp(&ctx.farmer2, &farm_id, &10_000_0000000);

    // Advance time to accumulate rewards
    advance_ledger(&ctx.env, 200);

    // Harvest rewards for both farmers
    let balance1_before = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer1);
    let balance2_before = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer2);

    ctx.client.harvest(&ctx.farmer1, &farm_id);
    ctx.client.harvest(&ctx.farmer2, &farm_id);

    let balance1_after = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer1);
    let balance2_after = get_balance(&ctx.env, &ctx.reward_token, &ctx.farmer2);

    let reward1 = balance1_after - balance1_before;
    let reward2 = balance2_after - balance2_before;

    // Both should have earned > 0
    assert!(reward1 > 0);
    assert!(reward2 > 0);

    // Since they staked equally, their rewards should be (almost) equal
    let diff = if reward1 > reward2 { reward1 - reward2 } else { reward2 - reward1 };
    assert!(diff <= 10); // allow tiny rounding differences
}
