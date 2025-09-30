use soroban_sdk::{panic_with_error, Address, Env, Symbol};
use crate::error::PoolError;
use crate::storage::{get_pool_info as storage_get_pool_info, set_pool_info, is_initialized, PoolInfo};

pub fn initialize(
    env: &Env,
    admin: Address,
    token_a: Address,
    token_b: Address,
    fee_rate: u32,
) {
    if is_initialized(env) {
        panic_with_error!(env, PoolError::AlreadyInitialized);
    }

    if fee_rate > 10000 { // Max 100% fee
        panic_with_error!(env, PoolError::InvalidFeeRate);
    }

    if token_a == token_b {
        panic_with_error!(env, PoolError::InvalidTokenPair);
    }

    let pool_info = PoolInfo {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
        reserve_a: 0,
        reserve_b: 0,
        total_lp_tokens: 0,
        fee_rate,
        admin,
        is_active: true,
    };

    set_pool_info(env, &pool_info);

    // Emit initialization event
    env.events().publish(
        (Symbol::new(env, "init"),),
        (token_a, token_b, fee_rate),
    );
}

pub fn get_pool_info(env: &Env) -> PoolInfo {
    storage_get_pool_info(env).unwrap_or_else(|| panic_with_error!(env, PoolError::NotInitialized))
}

pub fn get_reserves(env: &Env) -> (i128, i128) {
    let pool_info = get_pool_info(env);
    (pool_info.reserve_a, pool_info.reserve_b)
}

pub fn update_reserves(env: &Env, reserve_a: i128, reserve_b: i128) {
    let mut pool_info = get_pool_info(env);
    pool_info.reserve_a = reserve_a;
    pool_info.reserve_b = reserve_b;
    set_pool_info(env, &pool_info);
}

pub fn update_total_lp_tokens(env: &Env, total_lp_tokens: i128) {
    let mut pool_info = get_pool_info(env);
    pool_info.total_lp_tokens = total_lp_tokens;
    set_pool_info(env, &pool_info);
}

pub fn require_initialized(env: &Env) {
    if !is_initialized(env) {
        panic_with_error!(env, PoolError::NotInitialized);
    }
}

pub fn require_active(env: &Env) {
    let pool_info = get_pool_info(env);
    if !pool_info.is_active {
        panic_with_error!(env, PoolError::PoolNotActive);
    }
}

pub fn require_admin(env: &Env, caller: &Address) {
    let pool_info = get_pool_info(env);
    if pool_info.admin != *caller {
        panic_with_error!(env, PoolError::Unauthorized);
    }
}
