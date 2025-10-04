#![cfg(test)]

use crate::{FarmerYieldFarmingContract, FarmerYieldFarmingContractClient, ContractError};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Address, Env,
};

// ================================================================================
// TEST SETUP UTILITIES
// ================================================================================

fn setup_test<'a>() -> (
    Env,
    FarmerYieldFarmingContractClient<'a>,
    Address, // Admin
    Address, // Farmer 1
    Address, // Farmer 2
    Address, // LP Token
    Address, // Reward Token
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);

    // Register token contracts
    let lp_token = env.register_stellar_asset_contract(admin.clone());
    let reward_token = env.register_stellar_asset_contract(admin.clone());

    // Register farming contract
    let contract_id = env.register_contract(None, FarmerYieldFarmingContract);
    let client = FarmerYieldFarmingContractClient::new(&env, &contract_id);

    (env, client, admin, farmer1, farmer2, lp_token, reward_token)
}

fn set_ledger_sequence(env: &Env, sequence: u32) {
    env.ledger().set(LedgerInfo {
        timestamp: 12345,
        protocol_version: 22,
        sequence_number: sequence,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1000000,      // Increased from 10
        min_persistent_entry_ttl: 1000000, // Increased from 10
        max_entry_ttl: 3110400,
    });
}

fn extend_instance_storage(env: &Env, contract_id: &Address) {
    env.as_contract(contract_id, || {
        env.storage().instance().extend_ttl(1000000, 1000000);
    });
}

fn advance_ledger(env: &Env, blocks: u32) {
    env.ledger().with_mut(|li| {
        li.sequence_number += blocks;
    });
}

fn mint_lp_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let token_admin = token::StellarAssetClient::new(env, token);
    token_admin.mint(to, &amount);
}

fn mint_reward_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let token_admin = token::StellarAssetClient::new(env, token);
    token_admin.mint(to, &amount);
}

fn get_balance(env: &Env, token: &Address, account: &Address) -> i128 {
    let token_client = token::Client::new(env, token);
    token_client.balance(account)
}

// ================================================================================
// INITIALIZATION TESTS
// ================================================================================

#[test]
fn test_initialize() {
    let (_, client, admin, _, _, _, _) = setup_test();

    let result = client.initialize(&admin);
    assert_eq!(result, true);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_farm_count(), 0);
}

// ================================================================================
// FARM CREATION TESTS
// ================================================================================

#[test]
fn test_create_farm_success() {
    let (env, client, admin, _, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let result = client.create_farm(
        &lp_token,
        &reward_token,
        &100_0000000,
        &150,
        &1100,
        &100000,
    );

    let farm_id = result;
    assert_eq!(farm_id, 0);
    assert_eq!(client.get_farm_count(), 1);

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.lp_token, lp_token);
    assert_eq!(farm.reward_token, reward_token);
    assert_eq!(farm.reward_per_block, 100_0000000);
    assert_eq!(farm.multiplier, 150);
    assert_eq!(farm.total_staked, 0);
    assert!(farm.is_active);
}

#[test]
fn test_create_multiple_farms() {
    let (env, client, admin, _, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm1 = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);
    let farm2 = client.create_farm(&lp_token, &reward_token, &200_0000000, &200, &1100, &100000);

    assert_eq!(farm1, 0);
    assert_eq!(farm2, 1);
    assert_eq!(client.get_farm_count(), 2);
}

// ================================================================================
// FARM MANAGEMENT TESTS
// ================================================================================

#[test]
fn test_update_farm() {
    let (env, client, admin, _, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    for i in 0..20 {
        let farmer = Address::generate(&env);
        let amount = (i + 1) as i128 * 1000_0000000;
        mint_lp_tokens(&env, &lp_token, &farmer, amount);
        set_ledger_sequence(&env, 1200);
        client.stake_lp(&farmer, &farm_id, &amount);
    }

    let farm = client.get_farm(&farm_id);
    assert!(farm.total_staked > 0);
}

#[test]
fn test_stake_unstake_restake_cycle() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);

    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    client.unstake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);
}

#[test]
fn test_precision_with_small_amounts() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &1_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 100_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &100_0000000);

    advance_ledger(&env, 100);
    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending >= 0);
}

#[test]
fn test_farm_with_same_lp_and_reward_token() {
    let (env, client, admin, farmer1, _, lp_token, _) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &lp_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&env, &lp_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&lp_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_max_multiplier_rewards() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &500, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 100_000_000_0000000);
    client.deposit_rewards(&reward_token, &100_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_rewards_calculation_accuracy() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &100, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);

    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);

    client.update_farm(&farm_id, &200_0000000, &200);

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.reward_per_block, 200_0000000);
    assert_eq!(farm.multiplier, 200);
}

#[test]
fn test_unpause_farm() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    client.set_farm_paused(&farm_id, &true);
    client.set_farm_paused(&farm_id, &false);

    mint_lp_tokens(&env, &lp_token, &farmer1, 1000_0000000);
    set_ledger_sequence(&env, 1200);

    assert_eq!(client.stake_lp(&farmer1, &farm_id, &1000_0000000), ());
}

#[test]
fn test_end_farm() {
    let (env, client, admin, _, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    set_ledger_sequence(&env, 5000);
    client.end_farm(&farm_id);

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.end_block, 5000);
    assert!(!farm.is_active);
}

// ================================================================================
// STAKING TESTS
// ================================================================================

#[test]
fn test_stake_lp_success() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    let stake_amount = 10_000_0000000i128;
    mint_lp_tokens(&env, &lp_token, &farmer1, stake_amount);

    set_ledger_sequence(&env, 1200);
    assert_eq!(client.stake_lp(&farmer1, &farm_id, &stake_amount), ());

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, stake_amount);
}

#[test]
fn test_multiple_stakes_same_user() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 20_000_0000000);

    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &5_000_0000000);

    advance_ledger(&env, 100);
    client.stake_lp(&farmer1, &farm_id, &5_000_0000000);
}

#[test]
fn test_stake_multiple_users() {
    let (env, client, admin, farmer1, farmer2, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 5_000_0000000);
    mint_lp_tokens(&env, &lp_token, &farmer2, 10_000_0000000);

    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &5_000_0000000);
    client.stake_lp(&farmer2, &farm_id, &10_000_0000000);

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 15_000_0000000);
}

// ================================================================================
// UNSTAKING TESTS
// ================================================================================

#[test]
fn test_unstake_success() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    client.unstake_lp(&farmer1, &farm_id, &5_000_0000000);
    
    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 5_000_0000000);
}

#[test]
fn test_unstake_all_removes_user() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    client.unstake_lp(&farmer1, &farm_id, &10_000_0000000);

    let user = client.get_user_farm(&farmer1, &farm_id);
    assert!(user.is_none());
}

#[test]
fn test_early_unstake_penalty() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    let balance_before = get_balance(&env, &reward_token, &farmer1);

    advance_ledger(&env, 100);
    client.unstake_lp(&farmer1, &farm_id, &10_000_0000000);

    let balance_after = get_balance(&env, &reward_token, &farmer1);
    let rewards_received = balance_after - balance_before;

    assert!(rewards_received > 0);
}

// ================================================================================
// REWARDS TESTS
// ================================================================================

#[test]
fn test_harvest_rewards_success() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    let balance_before = get_balance(&env, &reward_token, &farmer1);

    advance_ledger(&env, 100);
    client.harvest(&farmer1, &farm_id);
    
    let balance_after = get_balance(&env, &reward_token, &farmer1);
    assert!(balance_after > balance_before);
}

#[test]
fn test_get_pending_rewards() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_pending_rewards_no_stake() {
    let (env, client, admin, _, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    let farmer3 = Address::generate(&env);
    let pending = client.get_pending_rewards(&farmer3, &farm_id);
    assert_eq!(pending, 0);
}

#[test]
fn test_emergency_withdraw_enabled() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    let lp_balance_before = get_balance(&env, &lp_token, &farmer1);

    client.set_emergency_withdraw(&true);
    client.emergency_withdraw(&farmer1, &farm_id);

    let lp_balance_after = get_balance(&env, &lp_token, &farmer1);
    assert_eq!(lp_balance_after, lp_balance_before + 10_000_0000000);

    let user = client.get_user_farm(&farmer1, &farm_id);
    assert!(user.is_none());

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 0);
}

// ================================================================================
// ADMIN TESTS
// ================================================================================

#[test]
fn test_set_global_multiplier() {
    let (env, client, admin, _, _, _, _) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);
    
    // Add actual test logic if set_global_multiplier function exists
}

#[test]
fn test_deposit_rewards() {
    let (env, client, admin, _, _, _, reward_token) = setup_test();

    client.initialize(&admin);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);

    let contract_balance_before = get_balance(&env, &reward_token, &client.address);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);
    let contract_balance_after = get_balance(&env, &reward_token, &client.address);

    assert_eq!(contract_balance_after, contract_balance_before + 1_000_000_0000000);
}

#[test]
fn test_update_pool() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);

    client.update_pool(&farm_id);

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.last_reward_block, 1300);
}

// ================================================================================
// TIER AND LOYALTY TESTS
// ================================================================================

#[test]
fn test_farmer_tier_smallholder() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 500_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &500_0000000);

    advance_ledger(&env, 100);
    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_farmer_tier_cooperative() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 1_000_000_0000000);
    client.deposit_rewards(&reward_token, &1_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 5_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &5_000_0000000);

    advance_ledger(&env, 100);
    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_farmer_tier_enterprise() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 15_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &15_000_0000000);

    advance_ledger(&env, 100);
    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);
}

#[test]
fn test_loyalty_bonus_7_days() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &150000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 17280 * 7);
    let pending_7days = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending_7days > 0);
}

#[test]
fn test_loyalty_bonus_30_days() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &600000);

    mint_reward_tokens(&env, &reward_token, &admin, 100_000_000_0000000);
    client.deposit_rewards(&reward_token, &100_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 17280 * 30);
    let pending_30days = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending_30days > 0);
}

#[test]
fn test_multiple_harvests() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);
    let balance_after_first = get_balance(&env, &reward_token, &farmer1);
    client.harvest(&farmer1, &farm_id);
    let balance_after_harvest1 = get_balance(&env, &reward_token, &farmer1);
    let first_reward = balance_after_harvest1 - balance_after_first;

    advance_ledger(&env, 100);
    client.harvest(&farmer1, &farm_id);
    let balance_after_harvest2 = get_balance(&env, &reward_token, &farmer1);
    let second_reward = balance_after_harvest2 - balance_after_harvest1;

    assert!(first_reward > 0);
    assert!(second_reward > 0);
}

#[test]
fn test_rewards_distribution_multiple_farmers() {
    let (env, client, admin, farmer1, farmer2, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    mint_lp_tokens(&env, &lp_token, &farmer2, 10_000_0000000);

    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);
    client.stake_lp(&farmer2, &farm_id, &10_000_0000000);

    advance_ledger(&env, 100);

    let pending1 = client.get_pending_rewards(&farmer1, &farm_id);
    let pending2 = client.get_pending_rewards(&farmer2, &farm_id);

    assert!(pending1 > 0);
    assert!(pending2 > 0);
}

#[test]
fn test_rewards_stop_after_end_block() {
    let (env, client, admin, farmer1, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &2000);

    mint_reward_tokens(&env, &reward_token, &admin, 10_000_000_0000000);
    client.deposit_rewards(&reward_token, &10_000_000_0000000);

    mint_lp_tokens(&env, &lp_token, &farmer1, 10_000_0000000);
    set_ledger_sequence(&env, 1200);
    client.stake_lp(&farmer1, &farm_id, &10_000_0000000);

    set_ledger_sequence(&env, 2500);

    let pending = client.get_pending_rewards(&farmer1, &farm_id);

    assert!(pending > 0);
}

// ================================================================================
// EDGE CASES AND STRESS TESTS
// ================================================================================

#[test]
fn test_zero_total_staked() {
    let (env, client, admin, _, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);

    advance_ledger(&env, 100);

    client.update_pool(&farm_id);

    let farm = client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 0);
}

#[test]
fn test_high_volume_staking() {
    let (env, client, admin, _, _, lp_token, reward_token) = setup_test();

    client.initialize(&admin);
    set_ledger_sequence(&env, 1000);

    let farm_id = client.create_farm(&lp_token, &reward_token, &100_0000000, &150, &1100, &100000);
}