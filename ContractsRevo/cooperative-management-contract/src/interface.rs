use crate::datatype::CooperativeError;
use soroban_sdk::{Address, Env, Map, String, Vec};

#[allow(dead_code)]
pub trait Membership {
    fn register_member(env: Env, member: Address, name: String, role: String) -> Result<(), CooperativeError>;
    fn verify_member(env: Env, admin: Address, address: Address) -> Result<(), CooperativeError>;
    fn track_contribution(env: Env, address: Address, amount: u32) -> Result<(), CooperativeError>;
    fn update_reputation(
        env: Env,
        admin: Address,
        address: Address,
        points: u32,
    ) -> Result<(), CooperativeError>;
}

#[allow(dead_code)]
pub trait ResourceSharing {
    fn register_resource(
        env: Env,
        owner: Address,
        description: String,
    ) -> Result<(), CooperativeError>;
    fn get_resources_by_owner(env: Env, owner: Address) -> Vec<u32>;
    fn borrow_resource(
        env: Env,
        borrower: Address,
        owner: Address,
        counter: u32,
    ) -> Result<(), CooperativeError>;
    fn return_resource(
        env: Env,
        caller: Address,
        owner: Address,
        counter: u32,
    ) -> Result<(), CooperativeError>;
    fn schedule_resource(
        env: Env,
        owner: Address,
        counter: u32,
        borrower: Address,
        time_slot: String,
    ) -> Result<(), CooperativeError>;
    fn track_maintenance(
        env: Env,
        owner: Address,
        caller: Address,
        resource_id: u32,
        details: String,
    ) -> Result<(), CooperativeError>;
}

#[allow(dead_code)]
pub trait ProfitDistribution {
    fn distribute_profits(
        env: Env,
        profits: i128,
        members: Vec<Address>,
    ) -> Result<Map<Address, i128>, CooperativeError>;
    fn share_expenses(
        env: Env,
        total_expense: i128,
        members: Vec<Address>,
    ) -> Result<Map<Address, i128>, CooperativeError>;
    fn pool_investment(env: Env, investor: Address, amount: i128) -> Result<(), CooperativeError>;
    fn process_automated_payments(
        env: Env,
        members: Vec<Address>,
        amount: i128,
    ) -> Result<(), CooperativeError>;
}

#[allow(dead_code)]
pub trait Governance {
    fn submit_proposal(
        env: Env,
        proposer: Address,
        proposal: String,
    ) -> Result<(), CooperativeError>;
    fn vote_on_proposal(
        env: Env,
        voter: Address,
        proposer: Address,
        approve: bool,
    ) -> Result<(), CooperativeError>;
    fn execute_decision(env: Env, proposer: Address) -> Result<(), CooperativeError>;
    fn trigger_emergency(env: Env, caller: Address, reason: String)
        -> Result<(), CooperativeError>;
    fn track_accountability(env: Env, member: Address) -> Result<i128, CooperativeError>;
}
