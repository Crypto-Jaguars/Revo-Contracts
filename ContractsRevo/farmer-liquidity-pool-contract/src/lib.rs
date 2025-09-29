#![no_std]

mod contract;
mod event;
mod interface;
mod storage;
mod types;
mod utils;
// If below wasm of lp-token-contact change then plese update this wasm also !
pub mod token {
    soroban_sdk::contractimport!(file = "./a_lp_token_contract.wasm");
}
