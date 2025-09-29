use soroban_sdk::{Address, ConversionError, Env, TryFromVal, Val};

use crate::types::{DataKey, Position};

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

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
