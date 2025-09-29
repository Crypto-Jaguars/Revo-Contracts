#![no_std]

mod contract;
mod event;
mod interface;
mod storage;
mod types;
mod utils;

// Note : If any changes made in lp-token-contract  , then don't forget to update the given wasm  fom new updates.
pub mod token {
    soroban_sdk::contractimport!(
        file = "./lp_token_contract.wasm"
    );
}
