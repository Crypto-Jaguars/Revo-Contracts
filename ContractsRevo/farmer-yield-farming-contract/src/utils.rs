#![cfg(test)]

use crate::{FarmerYieldFarmingContract, FarmerYieldFarmingContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token, Address, Env,
};

pub struct TestContext<'a> {
    pub env: Env,
    pub client: FarmerYieldFarmingContractClient<'a>,
    pub admin: Address,
    pub farmer1: Address,
    pub farmer2: Address,
    pub lp_token: Address,
    pub reward_token: Address,
}

pub fn setup_test<'a>() -> TestContext<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);

    // Register token contracts
    let lp_token = env.register_stellar_asset_contract_v2(admin.clone());
    let reward_token = env.register_stellar_asset_contract_v2(admin.clone());

    // Register farming contract
    let contract_id = env.register(FarmerYieldFarmingContract, ());
    let client = FarmerYieldFarmingContractClient::new(&env, &contract_id);

    TestContext {
        env,
        client,
        admin,
        farmer1,
        farmer2,
        lp_token: lp_token.address(),
        reward_token: reward_token.address(),
    }
}

pub fn set_ledger_sequence(env: &Env, sequence: u32) {
    env.ledger().set(LedgerInfo {
        timestamp: 12345,
        protocol_version: 22,
        sequence_number: sequence,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1000000,
        min_persistent_entry_ttl: 1000000,
        max_entry_ttl: 3110400,
    });
}

pub fn advance_ledger(env: &Env, blocks: u32) {
    env.ledger().with_mut(|li| {
        li.sequence_number += blocks;
    });
}

pub fn mint_lp_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let token_client = token::StellarAssetClient::new(env, token);
    token_client.mint(to, &amount);
}

pub fn mint_reward_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let token_client = token::StellarAssetClient::new(env, token);
    token_client.mint(to, &amount);
}

pub fn get_balance(env: &Env, token: &Address, account: &Address) -> i128 {
    let token_client = token::Client::new(env, token);
    token_client.balance(account)
}

pub fn setup_farm_with_rewards(ctx: &TestContext, reward_amount: i128) -> u32 {
    ctx.client.initialize(&ctx.admin);
    set_ledger_sequence(&ctx.env, 1000);

    let farm_id = ctx.client.create_farm(
        &ctx.lp_token,
        &ctx.reward_token,
        &100_0000000,
        &150,
        &1100,
        &100000,
    );

    if reward_amount > 0 {
        mint_reward_tokens(&ctx.env, &ctx.reward_token, &ctx.admin, reward_amount);
        ctx.client.deposit_rewards(&ctx.reward_token, &reward_amount);
    }

    farm_id
}