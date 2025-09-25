#![no_std]

mod dispute;
mod leasing;
mod payment;
mod utils;

pub use dispute::*;
pub use leasing::*;
pub use payment::*;
pub use utils::*;

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct LandLeasingContract;

#[contractimpl]
impl LandLeasingContract {
    /// Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address) {
        utils::set_admin(&env, &admin);
    }

    /// Create a new lease agreement
    pub fn create_lease(
        env: Env,
        lessor: Address,
        lessee: Address,
        land_id: soroban_sdk::BytesN<32>,
        location: soroban_sdk::String,
        size: u32,
        duration: u64,
        payment_amount: i128,
        data_hash: soroban_sdk::BytesN<32>,
    ) -> soroban_sdk::BytesN<32> {
        leasing::create_lease_agreement(
            &env,
            lessor,
            lessee,
            land_id,
            location,
            size,
            duration,
            payment_amount,
            data_hash,
        )
    }

    /// Process a lease payment
    pub fn process_payment(
        env: Env,
        lease_id: soroban_sdk::BytesN<32>,
        payer: Address,
        amount: i128,
    ) -> bool {
        payment::process_lease_payment(&env, lease_id, payer, amount)
    }

    /// Terminate a lease agreement
    pub fn terminate_lease(
        env: Env,
        lease_id: soroban_sdk::BytesN<32>,
        terminator: Address,
    ) -> bool {
        leasing::terminate_lease_agreement(&env, lease_id, terminator)
    }

    /// Resolve a dispute
    pub fn resolve_dispute(
        env: Env,
        lease_id: soroban_sdk::BytesN<32>,
        resolver: Address,
        resolution: soroban_sdk::String,
    ) -> bool {
        dispute::resolve_lease_dispute(&env, lease_id, resolver, resolution)
    }

    /// Get lease details
    pub fn get_lease_details(
        env: Env,
        lease_id: soroban_sdk::BytesN<32>,
    ) -> Option<leasing::LeaseAgreement> {
        leasing::get_lease_agreement(&env, lease_id)
    }

    /// Get land details
    pub fn get_land_details(env: Env, land_id: soroban_sdk::BytesN<32>) -> Option<leasing::Land> {
        leasing::get_land_info(&env, land_id)
    }

    /// Get payment history
    pub fn get_payment_history(
        env: Env,
        lease_id: soroban_sdk::BytesN<32>,
    ) -> soroban_sdk::Vec<payment::PaymentRecord> {
        payment::get_payment_history(&env, lease_id)
    }

    /// Raise a dispute
    pub fn raise_dispute(
        env: Env,
        lease_id: soroban_sdk::BytesN<32>,
        complainant: Address,
        reason: soroban_sdk::String,
    ) -> bool {
        dispute::raise_dispute(&env, lease_id, complainant, reason)
    }

    /// Extend lease duration
    pub fn extend_lease(
        env: Env,
        lease_id: soroban_sdk::BytesN<32>,
        requester: Address,
        additional_months: u64,
    ) -> bool {
        leasing::extend_lease_duration(&env, lease_id, requester, additional_months)
    }

    /// Get active leases for an address
    pub fn get_user_leases(env: Env, user: Address) -> soroban_sdk::Vec<soroban_sdk::BytesN<32>> {
        leasing::get_user_active_leases(&env, user)
    }
}

#[cfg(test)]
mod tests;
