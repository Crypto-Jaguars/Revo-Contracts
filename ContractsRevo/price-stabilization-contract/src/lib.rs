#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

mod datatype;
mod distribution;
mod fund;
mod interface;
mod pricing;
mod utils;

use crate::datatype::DataKey;

#[contract]
pub struct PriceStabilizationContract;

#[contractimpl]
impl PriceStabilizationContract {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Contract is already initialized");
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
