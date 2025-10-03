use crate::error::PoolError;
use crate::pool::{get_pool_info, require_active, require_initialized};
use crate::storage::{get_lp_balance as storage_get_lp_balance, set_lp_balance, set_pool_info};
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol};

// Simple square root implementation for i128
fn sqrt(env: &Env, n: i128) -> i128 {
    if n < 0 {
        panic_with_error!(env, PoolError::MathOverflow);
    }
    if n == 0 {
        return 0;
    }

    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

pub fn add_liquidity(
    env: &Env,
    provider: Address,
    amount_a: i128,
    amount_b: i128,
    min_lp_tokens: i128,
) -> i128 {
    require_initialized(env);
    require_active(env);

    if amount_a <= 0 || amount_b <= 0 {
        panic_with_error!(env, PoolError::InvalidAmount);
    }

    let mut pool_info = get_pool_info(env);

    // Transfer tokens from provider to contract
    token::Client::new(env, &pool_info.token_a).transfer(
        &provider,
        &env.current_contract_address(),
        &amount_a,
    );
    token::Client::new(env, &pool_info.token_b).transfer(
        &provider,
        &env.current_contract_address(),
        &amount_b,
    );

    let lp_tokens = if pool_info.total_lp_tokens == 0 {
        // First liquidity provision - use geometric mean
        let product = amount_a
            .checked_mul(amount_b)
            .unwrap_or_else(|| panic_with_error!(env, PoolError::MathOverflow));
        sqrt(env, product)
    } else {
        // Calculate LP tokens based on existing reserves
        let scaled_a = amount_a
            .checked_mul(pool_info.total_lp_tokens)
            .unwrap_or_else(|| panic_with_error!(env, PoolError::MathOverflow));
        let scaled_b = amount_b
            .checked_mul(pool_info.total_lp_tokens)
            .unwrap_or_else(|| panic_with_error!(env, PoolError::MathOverflow));
        let lp_tokens_a = scaled_a / pool_info.reserve_a;
        let lp_tokens_b = scaled_b / pool_info.reserve_b;

        // Use the smaller amount to maintain ratio
        if lp_tokens_a < lp_tokens_b {
            lp_tokens_a
        } else {
            lp_tokens_b
        }
    };

    if lp_tokens < min_lp_tokens {
        panic_with_error!(env, PoolError::SlippageExceeded);
    }

    // Update reserves and LP token supply
    pool_info.reserve_a += amount_a;
    pool_info.reserve_b += amount_b;
    pool_info.total_lp_tokens += lp_tokens;

    set_pool_info(env, &pool_info);

    // Update provider's LP token balance
    let current_balance = storage_get_lp_balance(env, &provider);
    set_lp_balance(env, &provider, current_balance + lp_tokens);

    // Emit liquidity added event
    env.events().publish(
        (Symbol::new(env, "liq_add"),),
        (provider, amount_a, amount_b, lp_tokens),
    );

    lp_tokens
}

pub fn remove_liquidity(
    env: &Env,
    provider: Address,
    lp_tokens: i128,
    min_amount_a: i128,
    min_amount_b: i128,
) -> (i128, i128) {
    require_initialized(env);
    require_active(env);

    if lp_tokens <= 0 {
        panic_with_error!(env, PoolError::InvalidLPTokenAmount);
    }

    let provider_balance = storage_get_lp_balance(env, &provider);
    if provider_balance < lp_tokens {
        panic_with_error!(env, PoolError::InsufficientBalance);
    }

    let mut pool_info = get_pool_info(env);

    if pool_info.total_lp_tokens == 0 {
        panic_with_error!(env, PoolError::InsufficientLiquidity);
    }

    // Calculate amounts to return
    let scaled_a = lp_tokens
        .checked_mul(pool_info.reserve_a)
        .unwrap_or_else(|| panic_with_error!(env, PoolError::MathOverflow));
    let scaled_b = lp_tokens
        .checked_mul(pool_info.reserve_b)
        .unwrap_or_else(|| panic_with_error!(env, PoolError::MathOverflow));
    let amount_a = scaled_a / pool_info.total_lp_tokens;
    let amount_b = scaled_b / pool_info.total_lp_tokens;

    if amount_a < min_amount_a || amount_b < min_amount_b {
        panic_with_error!(env, PoolError::SlippageExceeded);
    }

    if amount_a > pool_info.reserve_a || amount_b > pool_info.reserve_b {
        panic_with_error!(env, PoolError::InsufficientReserves);
    }

    // Update reserves and LP token supply
    pool_info.reserve_a -= amount_a;
    pool_info.reserve_b -= amount_b;
    pool_info.total_lp_tokens -= lp_tokens;

    set_pool_info(env, &pool_info);

    // Update provider's LP token balance
    set_lp_balance(env, &provider, provider_balance - lp_tokens);

    // Transfer tokens back to provider
    token::Client::new(env, &pool_info.token_a).transfer(
        &env.current_contract_address(),
        &provider,
        &amount_a,
    );
    token::Client::new(env, &pool_info.token_b).transfer(
        &env.current_contract_address(),
        &provider,
        &amount_b,
    );

    // Emit liquidity removed event
    env.events().publish(
        (Symbol::new(env, "liq_rem"),),
        (provider, lp_tokens, amount_a, amount_b),
    );

    (amount_a, amount_b)
}

pub fn get_lp_balance(env: &Env, provider: &Address) -> i128 {
    storage_get_lp_balance(env, provider)
}
