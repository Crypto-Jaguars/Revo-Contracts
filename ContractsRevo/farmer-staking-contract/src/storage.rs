use crate::errors::StakingError;
use crate::types::{PoolInfo, StakeInfo};
use soroban_sdk::{symbol_short, Address, Env, Symbol};

const POOL_INFO: Symbol = symbol_short!("POOL");
const STAKE_PREFIX: Symbol = symbol_short!("STAKE");

pub fn set_pool_info(env: &Env, pool_info: &PoolInfo) {
    env.storage().persistent().set(&POOL_INFO, pool_info);
}

pub fn get_pool_info(env: &Env) -> Result<PoolInfo, StakingError> {
    env.storage()
        .persistent()
        .get(&POOL_INFO)
        .ok_or(StakingError::PoolNotInitialized)
}

pub fn has_pool_info(env: &Env) -> bool {
    env.storage().persistent().has(&POOL_INFO)
}

pub fn set_stake_info(env: &Env, staker: &Address, stake_info: &StakeInfo) {
    let key = (STAKE_PREFIX, staker);
    env.storage().persistent().set(&key, stake_info);
}

pub fn get_stake_info(env: &Env, staker: &Address) -> Result<StakeInfo, StakingError> {
    let key = (STAKE_PREFIX, staker);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(StakingError::StakeNotFound)
}

pub fn has_stake_info(env: &Env, staker: &Address) -> bool {
    let key = (STAKE_PREFIX, staker);
    env.storage().persistent().has(&key)
}

pub fn remove_stake_info(env: &Env, staker: &Address) {
    let key = (STAKE_PREFIX, staker);
    env.storage().persistent().remove(&key);
}
