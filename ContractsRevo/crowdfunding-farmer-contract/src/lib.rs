#![no_std]

mod campaign;
mod contribution;
mod rewards;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

pub use campaign::{Campaign, CampaignStatus};
pub use contribution::Contribution;
pub use rewards::Reward;

#[contract]
pub struct CrowdfundingFarmerContract;

#[contractimpl]
impl CrowdfundingFarmerContract {
    pub fn create_campaign(
        env: Env,
        farmer_id: Address,
        goal_amount: i128,
        deadline: u64,
        reward_token: Address,
    ) -> BytesN<32> {
        campaign::create_campaign(env, farmer_id, goal_amount, deadline, reward_token)
    }

    pub fn contribute(env: Env, contributor: Address, campaign_id: BytesN<32>, amount: i128) {
        contribution::contribute(env, contributor, campaign_id, amount)
    }

    pub fn distribute_rewards(env: Env, campaign_id: BytesN<32>) {
        rewards::distribute_rewards(env, campaign_id)
    }

    pub fn refund_contributions(env: Env, campaign_id: BytesN<32>) {
        contribution::refund_contributions(env, campaign_id)
    }

    pub fn get_campaign_details(env: Env, campaign_id: BytesN<32>) -> Campaign {
        campaign::get_campaign_details(env, campaign_id)
    }

    pub fn get_contributions(env: Env, campaign_id: BytesN<32>) -> Vec<Contribution> {
        contribution::get_contributions(env, campaign_id)
    }
}

// #[cfg(test)]
// mod test;
