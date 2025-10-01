#![no_std]

use datatype::DataKey;
// Traits are implemented in separate modules
use soroban_sdk::{contract, contractimpl, Address, Env};

mod datatype;
mod governance;
mod interface;
mod membership;
mod profit_distribution;
mod resource_sharing;

#[cfg(test)]
mod original_tests;

#[cfg(test)]
mod tests;

#[contract]
pub struct CooperativeManagementContract;

#[contractimpl]
impl CooperativeManagementContract {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Contract is already initialized");
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }
}
