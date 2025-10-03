use crate::pool::{get_pool_info, require_active, require_initialized};
use crate::storage::{
    get_accumulated_fees as storage_get_accumulated_fees, get_lp_balance as storage_get_lp_balance,
    get_total_fees, set_accumulated_fees, set_total_fees,
};
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol};

pub fn claim_fees(env: &Env, provider: Address) -> (i128, i128) {
    require_initialized(env);
    require_active(env);

    let (fees_a, fees_b) = storage_get_accumulated_fees(env, &provider);

    if fees_a == 0 && fees_b == 0 {
        return (0, 0);
    }

    let mut pool_info = get_pool_info(env);

    // Check if we have sufficient reserves for the fee payout
    if fees_a > pool_info.reserve_a || fees_b > pool_info.reserve_b {
        panic_with_error!(env, crate::error::PoolError::InsufficientReserves);
    }

    // Transfer fees to provider
    if fees_a > 0 {
        token::Client::new(env, &pool_info.token_a).transfer(
            &env.current_contract_address(),
            &provider,
            &fees_a,
        );
    }

    if fees_b > 0 {
        token::Client::new(env, &pool_info.token_b).transfer(
            &env.current_contract_address(),
            &provider,
            &fees_b,
        );
    }

    // Decrease stored reserves to reflect the payout
    pool_info.reserve_a -= fees_a;
    pool_info.reserve_b -= fees_b;
    crate::storage::set_pool_info(env, &pool_info);

    // Reset accumulated fees for provider
    set_accumulated_fees(env, &provider, 0, 0);

    // Emit fee claim event
    env.events()
        .publish((Symbol::new(env, "fee_claim"),), (provider, fees_a, fees_b));

    (fees_a, fees_b)
}

pub fn get_accumulated_fees(env: &Env, provider: &Address) -> (i128, i128) {
    storage_get_accumulated_fees(env, provider)
}

pub fn distribute_fees(env: &Env) {
    require_initialized(env);

    let pool_info = get_pool_info(env);
    let (total_fees_a, total_fees_b) = get_total_fees(env);

    if total_fees_a == 0 && total_fees_b == 0 {
        return;
    }

    if pool_info.total_lp_tokens == 0 {
        // No liquidity providers to distribute fees to
        return;
    }

    // Distribute fees proportionally to LP token holders
    // For simplicity, we'll add fees to the pool reserves and let providers claim them
    // In a more sophisticated implementation, you would track individual provider shares

    // Add fees to pool reserves (they will be claimable by LP providers)
    let mut updated_pool_info = pool_info.clone();
    updated_pool_info.reserve_a += total_fees_a;
    updated_pool_info.reserve_b += total_fees_b;
    crate::storage::set_pool_info(env, &updated_pool_info);

    // Reset total fees as they are being distributed
    set_total_fees(env, 0, 0);

    // Emit fee distribution event
    env.events().publish(
        (Symbol::new(env, "fee_dist"),),
        (total_fees_a, total_fees_b, pool_info.total_lp_tokens),
    );
}

pub fn calculate_fee_share(env: &Env, provider: &Address, total_fees: i128) -> i128 {
    let pool_info = get_pool_info(env);
    let provider_lp_tokens = storage_get_lp_balance(env, provider);

    if pool_info.total_lp_tokens == 0 {
        return 0;
    }

    (provider_lp_tokens * total_fees) / pool_info.total_lp_tokens
}

pub fn add_fee_share(env: &Env, provider: &Address, fee_a: i128, fee_b: i128) {
    let (current_fees_a, current_fees_b) = get_accumulated_fees(env, provider);
    set_accumulated_fees(
        env,
        provider,
        current_fees_a + fee_a,
        current_fees_b + fee_b,
    );
}
