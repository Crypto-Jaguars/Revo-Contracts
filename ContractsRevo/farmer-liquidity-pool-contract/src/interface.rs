use crate::types::Position;
use soroban_sdk::{Address, BytesN, Env};

pub trait LiquidityPoolTrait {
    // Sets the token contract addresses for this pool
    fn initialize(
        e: Env,
        token_a: Address,
        token_b: Address,
        fee_rate: i128,
        token_wasm_hash: BytesN<32>,
    );

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn add_liquidity(
        e: Env,
        to: Address,
        desired_a: i128,
        min_a: i128,
        desired_b: i128,
        min_b: i128,
    );

    // If "buy_a" is true, the swap will buy token_a and sell token_b. This is flipped if "buy_a" is false.
    // "out" is the amount being bought, with in_max being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to "to".
    fn swap(e: Env, to: Address, buy_a: bool, out: i128, in_max: i128);

    // transfers share_amount of pool share tokens to this contract, burns all pools share tokens in this contracts, and sends the
    // corresponding amount of token_a and token_b to "to".
    // Returns amount of both tokens withdrawn
    fn remove_liquidity(
        e: Env,
        to: Address,
        share_amount: i128,
        min_a: i128,
        min_b: i128,
    ) -> (i128, i128);

    fn get_reserves(e: Env) -> (i128, i128);

    fn claim_fees(e: Env, provider: Address) -> (i128, i128);

    fn get_pending_fees(e: Env, provider: Address) -> (i128, i128);

    fn get_provider_position(e: Env, provider: Address) -> Option<Position>;
    fn collected_fees(e: Env) -> (i128, i128);
}
