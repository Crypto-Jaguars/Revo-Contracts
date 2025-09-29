use num_integer::Roots;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, IntoVal};

use crate::{event, interface::LiquidityPoolTrait, storage::*, token, types::Position, utils::*};

#[contract]
pub struct LiquidityPool;

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPool {
    fn initialize(
        e: Env,
        token_a: Address,
        token_b: Address,
        fee_rate: i128,
        token_wasm_hash: BytesN<32>,
    ) {
        if token_a >= token_b {
            panic!("token_a must be less than token_b");
        }
        if fee_rate <= 0 {
            panic!("Invalid fee rate value");
        }

        let share_contract = create_contract(&e, token_wasm_hash, &token_a, &token_b);
        token::Client::new(&e, &share_contract).initialize(
            &e.current_contract_address(),
            &7u32,
            &"Pool Share Token".into_val(&e),
            &"POOL".into_val(&e),
        );

        put_token_a(&e, token_a);
        put_token_b(&e, token_b);
        put_reserve_a(&e, 0);
        put_reserve_b(&e, 0);
        put_fee_rate(&e, fee_rate);
        put_token_share(&e, share_contract.try_into().unwrap());
        put_total_shares(&e, 0);
        put_accumulated_fee_a(&e, 0);
        put_accumulated_fee_b(&e, 0);
        put_fee_per_share_a(&e, 0);
        put_fee_per_share_b(&e, 0);
    }

    fn add_liquidity(
        e: Env,
        to: Address,
        desired_a: i128,
        min_a: i128,
        desired_b: i128,
        min_b: i128,
    ) {
        to.require_auth();

        let (reserve_a, reserve_b) = (get_reserve_a(&e), get_reserve_b(&e));
        let amounts = get_deposit_amounts(desired_a, min_a, desired_b, min_b, reserve_a, reserve_b);

        let token_a_client = token::Client::new(&e, &get_token_a(&e));
        let token_b_client = token::Client::new(&e, &get_token_b(&e));

        token_a_client.transfer(&to, &e.current_contract_address(), &amounts.0);
        token_b_client.transfer(&to, &e.current_contract_address(), &amounts.1);

        let (balance_a, balance_b) = (get_balance_a(&e), get_balance_b(&e));
        let total_shares = get_total_shares(&e);

        let zero = 0;
        let new_total_shares = if reserve_a > zero && reserve_b > zero {
            let shares_a = (balance_a * total_shares) / reserve_a;
            let shares_b = (balance_b * total_shares) / reserve_b;
            shares_a.min(shares_b)
        } else {
            (balance_a * balance_b).sqrt()
        };

        let shares_to_mint = new_total_shares - total_shares;

        // Update or create position for provider
        let current_fee_per_share_a = get_fee_per_share_a(&e);
        let current_fee_per_share_b = get_fee_per_share_b(&e);

        let position = if has_position(&e, &to) {
            let mut pos = get_provider_position(&e, to.clone());
            // Claim pending fees before updating position
            internal_claim_fees(&e, &mut pos);
            pos.shares += shares_to_mint;
            pos.liquidity += amounts.0 + amounts.1;
            pos.fee_debt_a = (pos.shares * current_fee_per_share_a) / 1_000_000_000;
            pos.fee_debt_b = (pos.shares * current_fee_per_share_b) / 1_000_000_000;
            pos
        } else {
            Position {
                provider: to.clone(),
                liquidity: amounts.0 + amounts.1,
                shares: shares_to_mint,
                fee_debt_a: (shares_to_mint * current_fee_per_share_a) / 1_000_000_000,
                fee_debt_b: (shares_to_mint * current_fee_per_share_b) / 1_000_000_000,
            }
        };

        put_provider_position(&e, to.clone(), position);
        mint_shares(&e, to.clone(), shares_to_mint);
        put_reserve_a(&e, balance_a);
        put_reserve_b(&e, balance_b);
        event::add_liquidity(&e, to, amounts.0, amounts.1);
    }

    fn swap(e: Env, to: Address, buy_a: bool, out: i128, in_max: i128) {
        to.require_auth();

        let (reserve_a, reserve_b) = (get_reserve_a(&e), get_reserve_b(&e));
        let (reserve_sell, reserve_buy) = if buy_a {
            (reserve_b, reserve_a)
        } else {
            (reserve_a, reserve_b)
        };

        let fee_rate = get_fee_rate(&e);
        let fee_numerator = 10000 - fee_rate;
        let fee_denominator = 10000;

        let n = reserve_sell * out * fee_denominator;
        let d = (reserve_buy - out) * fee_numerator;
        let sell_amount = (n / d) + 1;
        if sell_amount > in_max {
            panic!("in amount is over max")
        }

        // Calculate the fee collected from this swap
        let fee_collected = (sell_amount * fee_rate) / fee_denominator;

        let sell_token = if buy_a {
            get_token_b(&e)
        } else {
            get_token_a(&e)
        };
        let sell_token_client = token::Client::new(&e, &sell_token);
        sell_token_client.transfer(&to, &e.current_contract_address(), &sell_amount);

        let (balance_a, balance_b) = (get_balance_a(&e), get_balance_b(&e));

        let residue_numerator = fee_numerator;
        let residue_denominator = fee_denominator;
        let zero = 0;

        let new_invariant_factor = |balance: i128, reserve: i128, out: i128| {
            let delta = balance - reserve - out;
            let adj_delta = if delta > zero {
                residue_numerator * delta
            } else {
                residue_denominator * delta
            };
            residue_denominator * reserve + adj_delta
        };

        let (out_a, out_b) = if buy_a { (out, 0) } else { (0, out) };

        let new_inv_a = new_invariant_factor(balance_a, reserve_a, out_a);
        let new_inv_b = new_invariant_factor(balance_b, reserve_b, out_b);
        let old_inv_a = residue_denominator * reserve_a;
        let old_inv_b = residue_denominator * reserve_b;

        if new_inv_a * new_inv_b < old_inv_a * old_inv_b {
            panic!("constant product invariant does not hold");
        }

        if buy_a {
            transfer_a(&e, to.clone(), out_a);
        } else {
            transfer_b(&e, to.clone(), out_b);
        }

        put_reserve_a(&e, balance_a - out_a);
        put_reserve_b(&e, balance_b - out_b);

        distribute_fees(&e, buy_a, fee_collected);

        event::swap(&e, to, buy_a, sell_amount, out);
    }

    fn remove_liquidity(
        e: Env,
        to: Address,
        share_amount: i128,
        min_a: i128,
        min_b: i128,
    ) -> (i128, i128) {
        to.require_auth();

        if !has_position(&e, &to) {
            panic!("No position found for provider");
        }

        let mut position = get_provider_position(&e, to.clone());

        if position.shares < share_amount {
            panic!("Insufficient shares");
        }

        // Claim pending fees before removing liquidity
        internal_claim_fees(&e, &mut position);

        let share_token_client = token::Client::new(&e, &get_token_share(&e));
        share_token_client.transfer(&to, &e.current_contract_address(), &share_amount);

        let (balance_a, balance_b) = (get_balance_a(&e), get_balance_b(&e));
        let balance_shares = get_balance_shares(&e);
        let total_shares = get_total_shares(&e);

        let out_a = (balance_a * balance_shares) / total_shares;
        let out_b = (balance_b * balance_shares) / total_shares;

        if out_a < min_a || out_b < min_b {
            panic!("min not satisfied");
        }

        // Update position
        position.shares -= share_amount;
        position.liquidity -= out_a + out_b;

        let current_fee_per_share_a = get_fee_per_share_a(&e);
        let current_fee_per_share_b = get_fee_per_share_b(&e);
        position.fee_debt_a = (position.shares * current_fee_per_share_a) / 1_000_000_000;
        position.fee_debt_b = (position.shares * current_fee_per_share_b) / 1_000_000_000;

        if position.shares == 0 {
            remove_position(&e, &to);
        } else {
            put_provider_position(&e, to.clone(), position);
        }

        burn_shares(&e, balance_shares);
        transfer_a(&e, to.clone(), out_a);
        transfer_b(&e, to.clone(), out_b);
        put_reserve_a(&e, balance_a - out_a);
        put_reserve_b(&e, balance_b - out_b);

        event::remove_liquidity(&e, to, out_a, out_b);

        (out_a, out_b)
    }

    fn claim_fees(e: Env, provider: Address) -> (i128, i128) {
        provider.require_auth();

        if !has_position(&e, &provider) {
            panic!("No position found for provider");
        }

        let mut position = get_provider_position(&e, provider.clone());
        let (fee_a, fee_b) = internal_claim_fees(&e, &mut position);

        put_provider_position(&e, provider.clone(), position);

        // Transfer fees to provider
        if fee_a > 0 {
            transfer_a(&e, provider.clone(), fee_a);
        }
        if fee_b > 0 {
            transfer_b(&e, provider.clone(), fee_b);
        }

        event::fees_collected(&e, provider, fee_a, fee_b);

        (fee_a, fee_b)
    }

    fn get_reserves(e: Env) -> (i128, i128) {
        (get_reserve_a(&e), get_reserve_b(&e))
    }

    fn get_pending_fees(e: Env, provider: Address) -> (i128, i128) {
        if !has_position(&e, &provider) {
            return (0, 0);
        }

        let position = get_provider_position(&e, provider);
        let fee_per_share_a = get_fee_per_share_a(&e);
        let fee_per_share_b = get_fee_per_share_b(&e);

        let pending_a = ((position.shares * fee_per_share_a) / 1_000_000_000) - position.fee_debt_a;
        let pending_b = ((position.shares * fee_per_share_b) / 1_000_000_000) - position.fee_debt_b;

        (pending_a.max(0), pending_b.max(0))
    }

    fn get_provider_position(e: Env, provider: Address) -> Option<Position> {
        if has_position(&e, &provider) {
            Some(get_provider_position(&e, provider))
        } else {
            None
        }
    }

    fn collected_fees(e: Env) -> (i128, i128) {
        (get_accumulated_fee_a(&e), get_accumulated_fee_b(&e))
    }
}
