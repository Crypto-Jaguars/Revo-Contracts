#![no_std]

use datatype::DataKey;
use soroban_sdk::{contract, contractimpl, Address, Env};

mod interface;
mod datatype;
mod membership;
mod resource_sharing;
mod profit_distribution;
mod governance;

#[contract]
pub struct CooperativeManagementContract;

#[contractimpl]
impl CooperativeManagementContract {
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }
}
