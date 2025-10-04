use soroban_sdk::{
    contracttype, symbol_short, Address, ConversionError, Env, Symbol, TryFromVal, Val,
};

use crate::types::{DataKey, Position};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolInfo {
    pub token_a: Address,
    pub token_b: Address,
    pub reserve_a: i128,
    pub reserve_b: i128,
    pub total_lp_tokens: i128,
    pub fee_rate: u32, // Basis points (e.g., 30 = 0.3%)
    pub admin: Address,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityProvider {
    pub lp_tokens: i128,
    pub accumulated_fees_a: i128,
    pub accumulated_fees_b: i128,
    pub last_fee_claim: u64,
}

// Storage keys
const POOL_INFO: Symbol = symbol_short!("POOL_INFO");
const LP_BALANCES: Symbol = symbol_short!("LP_BAL");
const ACCUMULATED_FEES: Symbol = symbol_short!("ACC_FEES");
const TOTAL_FEES: Symbol = symbol_short!("TOT_FEES");

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

// Pool info functions
pub fn get_pool_info(env: &Env) -> Option<PoolInfo> {
    env.storage().persistent().get(&POOL_INFO)
}

pub fn set_pool_info(env: &Env, pool_info: &PoolInfo) {
    env.storage().persistent().set(&POOL_INFO, pool_info);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&POOL_INFO)
}

// LP balance functions
pub fn get_lp_balance(env: &Env, provider: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&(LP_BALANCES, provider))
        .unwrap_or(0)
}

pub fn set_lp_balance(env: &Env, provider: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&(LP_BALANCES, provider), &amount);
}

// Fee functions
pub fn get_accumulated_fees(env: &Env, provider: &Address) -> (i128, i128) {
    let fees: Option<(i128, i128)> = env
        .storage()
        .persistent()
        .get(&(ACCUMULATED_FEES, provider));
    fees.unwrap_or((0, 0))
}

pub fn set_accumulated_fees(env: &Env, provider: &Address, fees_a: i128, fees_b: i128) {
    env.storage()
        .persistent()
        .set(&(ACCUMULATED_FEES, provider), &(fees_a, fees_b));
}

pub fn get_total_fees(env: &Env) -> (i128, i128) {
    env.storage()
        .persistent()
        .get(&TOTAL_FEES)
        .unwrap_or((0, 0))
}

pub fn set_total_fees(env: &Env, fees_a: i128, fees_b: i128) {
    env.storage()
        .persistent()
        .set(&TOTAL_FEES, &(fees_a, fees_b));
}

// Legacy functions for compatibility with the other implementation
pub fn get_token_a(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenA).unwrap()
}

pub fn get_token_b(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenB).unwrap()
}

pub fn get_token_share(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenShare).unwrap()
}

pub fn get_total_shares(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalShares).unwrap()
}

pub fn get_reserve_a(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::ReserveA).unwrap()
}

pub fn get_reserve_b(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::ReserveB).unwrap()
}

pub fn get_fee_rate(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::FeeRate).unwrap()
}

pub fn get_provider_position(e: &Env, provider: Address) -> Position {
    e.storage().instance().get(&provider).unwrap()
}

pub fn get_accumulated_fee_a(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::AccumulatedFeeA)
        .unwrap_or(0)
}

pub fn get_accumulated_fee_b(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::AccumulatedFeeB)
        .unwrap_or(0)
}

pub fn get_fee_per_share_a(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::FeePerShareA)
        .unwrap_or(0)
}

pub fn get_fee_per_share_b(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::FeePerShareB)
        .unwrap_or(0)
}

pub fn put_token_a(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::TokenA, &contract);
}

pub fn put_token_b(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::TokenB, &contract);
}

pub fn put_reserve_a(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::ReserveA, &amount)
}

pub fn put_reserve_b(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::ReserveB, &amount)
}

pub fn put_fee_rate(e: &Env, rate: i128) {
    e.storage().instance().set(&DataKey::FeeRate, &rate);
}

pub fn put_provider_position(e: &Env, provider: Address, position: Position) {
    e.storage().instance().set(&provider, &position);
}

pub fn put_token_share(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::TokenShare, &contract);
}

pub fn put_total_shares(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalShares, &amount)
}

pub fn put_accumulated_fee_a(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::AccumulatedFeeA, &amount);
}

pub fn put_accumulated_fee_b(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::AccumulatedFeeB, &amount);
}

pub fn put_fee_per_share_a(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::FeePerShareA, &amount);
}

pub fn put_fee_per_share_b(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::FeePerShareB, &amount);
}

pub fn remove_position(e: &Env, provider: &Address) {
    e.storage().instance().remove(provider);
}

pub fn has_position(e: &Env, provider: &Address) -> bool {
    e.storage().instance().has(provider)
}
