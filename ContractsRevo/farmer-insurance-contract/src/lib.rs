
#![no_std]

use soroban_sdk::{contract, contractimpl, BytesN, Address, Symbol, Env};

mod utils;
mod insurance;
mod claims;
mod payouts;

#[contract]
pub struct FarmerInsuranceContract;

#[contractimpl]
impl FarmerInsuranceContract {
    pub fn create_pol(env: Env, farmer: Address, coverage: Symbol, premium: i128) -> BytesN<32> {
        insurance::create_pol(env, farmer, coverage, premium)
    }

    pub fn pay_prem(env: Env, policy_id: BytesN<32>) {
        insurance::pay_prem(env, policy_id)
    }

    pub fn sub_claim(env: Env, policy_id: BytesN<32>, event_hash: BytesN<32>, payout: i128) -> BytesN<32> {
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
mod test;