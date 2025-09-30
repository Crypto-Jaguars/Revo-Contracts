use soroban_sdk::{panic_with_error, Address, Env, Symbol, token};
use crate::error::PoolError;
use crate::storage::{set_pool_info, get_total_fees, set_total_fees};
use crate::pool::{require_initialized, require_active, get_pool_info};

pub fn execute_swap(
    env: &Env,
    trader: Address,
    token_in: Address,
    amount_in: i128,
    min_amount_out: i128,
) -> i128 {
    require_initialized(env);
    require_active(env);

    if amount_in <= 0 {
        panic_with_error!(env, PoolError::InvalidAmount);
    }

    let mut pool_info = get_pool_info(env);
    
    // Determine which token is being swapped in
    let (token_out, reserve_in, reserve_out) = if token_in == pool_info.token_a {
        (pool_info.token_b.clone(), pool_info.reserve_a, pool_info.reserve_b)
    } else if token_in == pool_info.token_b {
        (pool_info.token_a.clone(), pool_info.reserve_b, pool_info.reserve_a)
    } else {
        panic_with_error!(env, PoolError::InvalidToken);
    };

    if reserve_in == 0 || reserve_out == 0 {
        panic_with_error!(env, PoolError::InsufficientLiquidity);
    }

    // Calculate swap output using constant product formula
    let amount_out = calculate_swap_output_internal(env, amount_in, reserve_in, reserve_out, pool_info.fee_rate);

    if amount_out < min_amount_out {
        panic_with_error!(env, PoolError::SlippageExceeded);
    }

    if amount_out >= reserve_out {
        panic_with_error!(env, PoolError::InsufficientReserves);
    }

    // Transfer tokens
    token::Client::new(env, &token_in).transfer(&trader, &env.current_contract_address(), &amount_in);
    token::Client::new(env, &token_out).transfer(&env.current_contract_address(), &trader, &amount_out);

    // Update reserves
    if token_in == pool_info.token_a {
        pool_info.reserve_a += amount_in;
        pool_info.reserve_b -= amount_out;
    } else {
        pool_info.reserve_a -= amount_out;
        pool_info.reserve_b += amount_in;
    }

    set_pool_info(env, &pool_info);

    // Update total fees collected
    let fee_amount = (amount_in * pool_info.fee_rate as i128) / 10000;
    let (total_fees_a, total_fees_b) = get_total_fees(env);
    
    if token_in == pool_info.token_a {
        set_total_fees(env, total_fees_a + fee_amount, total_fees_b);
    } else {
        set_total_fees(env, total_fees_a, total_fees_b + fee_amount);
    }

    // Emit swap event
    env.events().publish(
        (Symbol::new(env, "swap"),),
        (trader, token_in, amount_in, token_out, amount_out),
    );

    amount_out
}

pub fn calculate_swap_output(
    env: &Env,
    token_in: Address,
    amount_in: i128,
) -> i128 {
    require_initialized(env);

    let pool_info = get_pool_info(env);
    
    let (reserve_in, reserve_out) = if token_in == pool_info.token_a {
        (pool_info.reserve_a, pool_info.reserve_b)
    } else if token_in == pool_info.token_b {
        (pool_info.reserve_b, pool_info.reserve_a)
    } else {
        panic_with_error!(env, PoolError::InvalidToken);
    };

    if reserve_in == 0 || reserve_out == 0 {
        return 0;
    }

    calculate_swap_output_internal(env, amount_in, reserve_in, reserve_out, pool_info.fee_rate)
}

fn calculate_swap_output_internal(
    env: &Env,
    amount_in: i128,
    reserve_in: i128,
    reserve_out: i128,
    fee_rate: u32,
) -> i128 {
    // Apply fee to input amount
    let fee_amount = (amount_in * fee_rate as i128) / 10000;
    let amount_in_after_fee = amount_in - fee_amount;

    // Constant product formula: (reserve_in + amount_in_after_fee) * (reserve_out - amount_out) = reserve_in * reserve_out
    // Solving for amount_out: amount_out = (amount_in_after_fee * reserve_out) / (reserve_in + amount_in_after_fee)
    
    let numerator = amount_in_after_fee.checked_mul(reserve_out)
        .unwrap_or_else(|| panic_with_error!(env, PoolError::MathOverflow));
    let denominator = reserve_in + amount_in_after_fee;
    
    if denominator == 0 {
        panic_with_error!(env, PoolError::DivisionByZero);
    }
    
    numerator / denominator
}
