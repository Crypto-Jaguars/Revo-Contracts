#![no_std]

use datatype::DataKey;
use soroban_sdk::{Address, Env, contract, contractimpl};

mod datatype;
mod governance;
mod interface;
mod membership;
mod profit_distribution;
mod resource_sharing;

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

// Implement Membership trait
impl interface::Membership for CooperativeManagementContract {
    fn register_member(env: Env, member: Address, name: String) -> Result<(), datatype::CooperativeError> {
        membership::Membership::register_member(env, member, name)
    }

    fn verify_member(env: Env, admin: Address, address: Address) -> Result<(), datatype::CooperativeError> {
        membership::Membership::verify_member(env, admin, address)
    }

    fn track_contribution(env: Env, address: Address, amount: u32) -> Result<(), datatype::CooperativeError> {
        membership::Membership::track_contribution(env, address, amount)
    }

    fn update_reputation(env: Env, admin: Address, address: Address, points: u32) -> Result<(), datatype::CooperativeError> {
        membership::Membership::update_reputation(env, admin, address, points)
    }
}

// Implement ResourceSharing trait
impl interface::ResourceSharing for CooperativeManagementContract {
    fn register_resource(env: Env, owner: Address, description: String) -> Result<(), datatype::CooperativeError> {
        resource_sharing::ResourceSharing::register_resource(env, owner, description)
    }

    fn get_resources_by_owner(env: Env, owner: Address) -> soroban_sdk::Vec<u32> {
        resource_sharing::ResourceSharing::get_resources_by_owner(env, owner)
    }

    fn borrow_resource(env: Env, borrower: Address, owner: Address, counter: u32) -> Result<(), datatype::CooperativeError> {
        resource_sharing::ResourceSharing::borrow_resource(env, borrower, owner, counter)
    }

    fn return_resource(env: Env, caller: Address, owner: Address, counter: u32) -> Result<(), datatype::CooperativeError> {
        resource_sharing::ResourceSharing::return_resource(env, caller, owner, counter)
    }

    fn schedule_resource(env: Env, owner: Address, counter: u32, borrower: Address, time_slot: String) -> Result<(), datatype::CooperativeError> {
        resource_sharing::ResourceSharing::schedule_resource(env, owner, counter, borrower, time_slot)
    }

    fn track_maintenance(env: Env, owner: Address, caller: Address, resource_id: u32, details: String) -> Result<(), datatype::CooperativeError> {
        resource_sharing::ResourceSharing::track_maintenance(env, owner, caller, resource_id, details)
    }
}

// Implement Governance trait
impl interface::Governance for CooperativeManagementContract {
    fn submit_proposal(env: Env, proposer: Address, proposal: String) -> Result<(), datatype::CooperativeError> {
        governance::Governance::submit_proposal(env, proposer, proposal)
    }

    fn vote_on_proposal(env: Env, voter: Address, proposer: Address, approve: bool) -> Result<(), datatype::CooperativeError> {
        governance::Governance::vote_on_proposal(env, voter, proposer, approve)
    }

    fn execute_decision(env: Env, proposer: Address) -> Result<(), datatype::CooperativeError> {
        governance::Governance::execute_decision(env, proposer)
    }

    fn trigger_emergency(env: Env, caller: Address, reason: String) -> Result<(), datatype::CooperativeError> {
        governance::Governance::trigger_emergency(env, caller, reason)
    }

    fn track_accountability(env: Env, member: Address) -> Result<i128, datatype::CooperativeError> {
        governance::Governance::track_accountability(env, member)
    }
}

#[cfg(test)]
mod test;
