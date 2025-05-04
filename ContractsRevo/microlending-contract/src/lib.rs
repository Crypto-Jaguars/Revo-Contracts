#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

mod datatypes;
mod fund;
mod request;

pub use datatypes::*;
pub use fund::*;
pub use request::*;

#[contract]
pub struct Microlending;

#[contractimpl]
impl Microlending {
    // Initialize the contract
    // pub fn initialize(env: Env, admin: Address) {
    //     admin.require_auth();
    //     env.storage().instance().set(&DataKey::Admin, &admin);
    //     env.storage().instance().set(&DataKey::NextProductId, &1u32);
    // }

    // // Product functions
    // pub fn create_product(
    //     env: Env,
    //     creator: Address,
    //     name: String,
    //     description: String,
    //     funding_goal: u64,
    //     deadline: u64, // Changed from &u64
    //     reward_tiers: Vec<RewardTier>,
    //     milestones: Vec<Milestone>,
    // ) -> u32 {
    //     product::create_product(
    //         env,
    //         creator,
    //         name,
    //         description,
    //         funding_goal,
    //         deadline,
    //         reward_tiers,
    //         milestones,
    //     )
    // }

    // // Funding functions
    // pub fn contribute(env: Env, contributor: Address, product_id: u32, amount: u64) {
    //     funding::contribute(env, contributor, product_id, amount)
    // }

    // pub fn distribute_funds(env: Env, product_id: u32) {
    //     funding::distribute_funds(env, product_id)
    // }

    // pub fn refund_contributors(env: Env, product_id: u32) {
    //     funding::refund_contributors(env, product_id)
    // }

    // // Reward functions
    // pub fn claim_reward(env: Env, contributor: Address, product_id: u32) {
    //     rewards::claim_reward(env, contributor, product_id)
    // }

    // // Tracking functions
    // pub fn update_milestone(env: Env, creator: Address, product_id: u32, milestone_id: u32) {
    //     tracking::update_milestone(env, creator, product_id, milestone_id)
    // }

    // pub fn get_product(env: Env, product_id: u32) -> Product {
    //     product::get_product(env, product_id)
    // }

    // pub fn get_contributions(env: Env, product_id: u32) -> Vec<Contribution> {
    //     tracking::get_contributions(env, product_id)
    // }

    // pub fn get_milestones(env: Env, product_id: u32) -> Vec<Milestone> {
    //     tracking::get_milestones(env, product_id)
    // }

    // pub fn get_reward_tiers(env: Env, product_id: u32) -> Vec<RewardTier> {
    //     rewards::get_reward_tiers(env, product_id)
    // }
}
