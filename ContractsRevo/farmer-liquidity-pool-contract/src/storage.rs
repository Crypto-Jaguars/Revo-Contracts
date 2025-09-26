use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

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

pub fn get_pool_info(env: &Env) -> Option<PoolInfo> {
    env.storage().persistent().get(&POOL_INFO)
}

pub fn set_pool_info(env: &Env, pool_info: &PoolInfo) {
    env.storage().persistent().set(&POOL_INFO, pool_info);
}

pub fn get_lp_balance(env: &Env, provider: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&(LP_BALANCES, provider))
        .unwrap_or(0)
}

pub fn set_lp_balance(env: &Env, provider: &Address, amount: i128) {
    env.storage().persistent().set(&(LP_BALANCES, provider), &amount);
}

pub fn get_accumulated_fees(env: &Env, provider: &Address) -> (i128, i128) {
    let fees: Option<(i128, i128)> = env.storage()
        .persistent()
        .get(&(ACCUMULATED_FEES, provider));
    fees.unwrap_or((0, 0))
}

pub fn set_accumulated_fees(env: &Env, provider: &Address, fees_a: i128, fees_b: i128) {
    env.storage().persistent().set(&(ACCUMULATED_FEES, provider), &(fees_a, fees_b));
}

pub fn get_total_fees(env: &Env) -> (i128, i128) {
    env.storage()
        .persistent()
        .get(&TOTAL_FEES)
        .unwrap_or((0, 0))
}

pub fn set_total_fees(env: &Env, fees_a: i128, fees_b: i128) {
    env.storage().persistent().set(&TOTAL_FEES, &(fees_a, fees_b));
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().persistent().has(&POOL_INFO)
}
