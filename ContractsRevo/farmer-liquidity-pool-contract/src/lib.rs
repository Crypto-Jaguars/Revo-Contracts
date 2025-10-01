#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[cfg(test)]
mod tests;

mod pool;
mod liquidity;
mod swap;
mod fees;
mod error;
mod storage;
mod event;
mod interface;
mod types;
mod utils;

pub use pool::*;
pub use liquidity::*;
pub use swap::*;
pub use fees::*;
pub use error::*;
pub use storage::{PoolInfo, LiquidityProvider, is_initialized, set_pool_info};

// If below wasm of lp-token-contact change then plese update this wasm also !
pub mod token {
    soroban_sdk::contractimport!(file = "./a_lp_token_contract.wasm");
}

#[contract]
pub struct FarmerLiquidityPoolContract;

#[contractimpl]
impl FarmerLiquidityPoolContract {
    /// Initialize the liquidity pool contract
    pub fn initialize(
        env: Env,
        admin: Address,
        token_a: Address,
        token_b: Address,
        fee_rate: u32, // Fee rate in basis points (e.g., 30 = 0.3%)
    ) {
        pool::initialize(&env, admin, token_a, token_b, fee_rate);
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        env: Env,
        provider: Address,
        amount_a: i128,
        amount_b: i128,
        min_lp_tokens: i128,
    ) -> i128 {
        liquidity::add_liquidity(&env, provider, amount_a, amount_b, min_lp_tokens)
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        env: Env,
        provider: Address,
        lp_tokens: i128,
        min_amount_a: i128,
        min_amount_b: i128,
    ) -> (i128, i128) {
        liquidity::remove_liquidity(&env, provider, lp_tokens, min_amount_a, min_amount_b)
    }

    /// Swap tokens using constant product formula
    pub fn swap(
        env: Env,
        trader: Address,
        token_in: Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> i128 {
        swap::execute_swap(&env, trader, token_in, amount_in, min_amount_out)
    }

    /// Claim accumulated fees for a liquidity provider
    pub fn claim_fees(env: Env, provider: Address) -> (i128, i128) {
        fees::claim_fees(&env, provider)
    }

    /// Get pool information
    pub fn get_pool_info(env: Env) -> PoolInfo {
        pool::get_pool_info(&env)
    }

    /// Get liquidity provider's LP token balance
    pub fn get_lp_balance(env: Env, provider: Address) -> i128 {
        liquidity::get_lp_balance(&env, &provider)
    }

    /// Get reserves for both tokens
    pub fn get_reserves(env: Env) -> (i128, i128) {
        pool::get_reserves(&env)
    }

    /// Calculate swap output amount
    pub fn calculate_swap_output(
        env: Env,
        token_in: Address,
        amount_in: i128,
    ) -> i128 {
        swap::calculate_swap_output(&env, token_in, amount_in)
    }

    /// Get accumulated fees for a provider
    pub fn get_accumulated_fees(env: Env, provider: Address) -> (i128, i128) {
        fees::get_accumulated_fees(&env, &provider)
    }

    /// Distribute accumulated fees to liquidity providers
    pub fn distribute_fees(env: Env) {
        fees::distribute_fees(&env);
    }

    /// Calculate fee share for a provider
    pub fn calculate_fee_share(env: Env, provider: Address, total_fees: i128) -> i128 {
        fees::calculate_fee_share(&env, &provider, total_fees)
    }
}
