#![cfg(test)]

use crate::FarmerInsuranceContract;
use soroban_sdk::{testutils::Address as _, Address, Env};

pub fn create_test_contract(env: &Env) -> Address {
    env.register(FarmerInsuranceContract, ())
}

pub fn create_test_accounts(env: &Env) -> (Address, Address) {
    (Address::generate(env), Address::generate(env))
}
