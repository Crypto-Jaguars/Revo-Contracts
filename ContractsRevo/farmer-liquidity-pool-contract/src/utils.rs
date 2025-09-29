use crate::{storage::*, token, types::Position};
use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

pub fn get_balance(e: &Env, contract: Address) -> i128 {
    token::Client::new(e, &contract).balance(&e.current_contract_address())
}

pub fn get_balance_a(e: &Env) -> i128 {
    get_balance(e, get_token_a(e))
}

pub fn get_balance_b(e: &Env) -> i128 {
    get_balance(e, get_token_b(e))
}

pub fn get_balance_shares(e: &Env) -> i128 {
    get_balance(e, get_token_share(e))
}

pub fn transfer(e: &Env, token: Address, to: Address, amount: i128) {
    token::Client::new(e, &token).transfer(&e.current_contract_address(), &to, &amount);
}

pub fn transfer_a(e: &Env, to: Address, amount: i128) {
    transfer(e, get_token_a(e), to, amount);
}

pub fn transfer_b(e: &Env, to: Address, amount: i128) {
    transfer(e, get_token_b(e), to, amount);
}

pub fn get_deposit_amounts(
    desired_a: i128,
    min_a: i128,
    desired_b: i128,
    min_b: i128,
    reserve_a: i128,
    reserve_b: i128,
) -> (i128, i128) {
    if reserve_a == 0 && reserve_b == 0 {
        return (desired_a, desired_b);
    }

    let amount_b = desired_a * reserve_b / reserve_a;
    if amount_b <= desired_b {
        if amount_b < min_b {
            panic!("amount_b less than min")
        }
        (desired_a, amount_b)
    } else {
        let amount_a = desired_b * reserve_a / reserve_b;
        if amount_a > desired_a || desired_a < min_a {
            panic!("amount_a invalid")
        }
        (amount_a, desired_b)
    }
}

pub fn create_contract(
    e: &Env,
    token_wasm_hash: BytesN<32>,
    token_a: &Address,
    token_b: &Address,
) -> Address {
    let mut salt = Bytes::new(e);
    salt.append(&token_a.to_xdr(e));
    salt.append(&token_b.to_xdr(e));
    let salt = e.crypto().sha256(&salt);
    e.deployer()
        .with_current_contract(salt)
        .deploy_v2(token_wasm_hash, ())
}

pub fn burn_shares(e: &Env, amount: i128) {
    let total = get_total_shares(e);
    let share_contract = get_token_share(e);

    token::Client::new(e, &share_contract).burn(&e.current_contract_address(), &amount);
    put_total_shares(e, total - amount);
}

pub fn mint_shares(e: &Env, to: Address, amount: i128) {
    let total = get_total_shares(e);
    let share_contract_id = get_token_share(e);

    token::Client::new(e, &share_contract_id).mint(&to, &amount);

    put_total_shares(e, total + amount);
}

pub fn distribute_fees(e: &Env, is_token_b_fee: bool, fee_amount: i128) {
    let total_shares = get_total_shares(e);

    if total_shares == 0 {
        return;
    }

    // Fee per share with precision multiplier (1e9)
    let fee_per_share = (fee_amount * 1_000_000_000) / total_shares;

    if is_token_b_fee {
        let current_fee_per_share_b = get_fee_per_share_b(e);
        put_fee_per_share_b(e, current_fee_per_share_b + fee_per_share);
        let accumulated = get_accumulated_fee_b(e);
        put_accumulated_fee_b(e, accumulated + fee_amount);
    } else {
        let current_fee_per_share_a = get_fee_per_share_a(e);
        put_fee_per_share_a(e, current_fee_per_share_a + fee_per_share);
        let accumulated = get_accumulated_fee_a(e);
        put_accumulated_fee_a(e, accumulated + fee_amount);
    }
}

pub fn internal_claim_fees(e: &Env, position: &mut Position) -> (i128, i128) {
    let fee_per_share_a = get_fee_per_share_a(e);
    let fee_per_share_b = get_fee_per_share_b(e);

    let pending_a = ((position.shares * fee_per_share_a) / 1_000_000_000) - position.fee_debt_a;
    let pending_b = ((position.shares * fee_per_share_b) / 1_000_000_000) - position.fee_debt_b;

    let fee_a = pending_a.max(0);
    let fee_b = pending_b.max(0);

    // Update fee debts
    position.fee_debt_a = (position.shares * fee_per_share_a) / 1_000_000_000;
    position.fee_debt_b = (position.shares * fee_per_share_b) / 1_000_000_000;

    // Update accumulated fees
    if fee_a > 0 {
        let accumulated = get_accumulated_fee_a(e);
        put_accumulated_fee_a(e, accumulated - fee_a);
    }
    if fee_b > 0 {
        let accumulated = get_accumulated_fee_b(e);
        put_accumulated_fee_b(e, accumulated - fee_b);
    }

    (fee_a, fee_b)
}
