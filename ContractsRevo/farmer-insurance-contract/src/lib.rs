#![no_std]

use crate::utils::ContractError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol};

mod claims;
mod insurance;
mod payouts;
mod utils;

#[contract]
pub struct FarmerInsuranceContract;

#[contractimpl]
impl FarmerInsuranceContract {
    pub fn create_pol(
        env: Env,
        farmer: Address,
        coverage: Symbol,
        premium: i128,
    ) -> Result<BytesN<32>, ContractError> {
        insurance::create_pol(env, farmer, coverage, premium)
    }

    pub fn pay_prem(env: Env, policy_id: BytesN<32>) {
        insurance::pay_prem(env, policy_id)
    }

    pub fn sub_claim(
        env: Env,
        policy_id: BytesN<32>,
        event_hash: BytesN<32>,
        payout: i128,
    ) -> Result<BytesN<32>, ContractError> {
        claims::sub_claim(env, policy_id, event_hash, payout)
    }

    pub fn pay_out(env: Env, claim_id: BytesN<32>, admin: Address) {
        payouts::pay_out(env, claim_id, admin)
    }

    pub fn get_policy(env: Env, policy_id: BytesN<32>) -> insurance::InsurancePolicy {
        insurance::get_policy(env, policy_id)
    }
}

#[cfg(test)]
mod tests;
