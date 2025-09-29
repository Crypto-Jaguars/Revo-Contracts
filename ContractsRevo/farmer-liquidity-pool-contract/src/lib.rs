#![no_std]

mod contract;
mod event;
mod interface;
mod storage;
mod types;
mod utils;

pub mod token {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/a_lp_token_contract.wasm"
    );
}
