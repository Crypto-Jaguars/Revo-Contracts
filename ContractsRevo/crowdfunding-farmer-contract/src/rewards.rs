use soroban_sdk::{Address, BytesN, Env};

use crate::{contribution, utils, CampaignStatus};

#[derive(Clone)]
pub struct Reward {
    pub contributor_id: Address,
    pub campaign_id: BytesN<32>,
    pub amount: i128,
}

pub fn distribute_rewards(env: Env, campaign_id: BytesN<32>) {
    // Get campaign
    let campaign = utils::read_campaign(&env, &campaign_id)
        .unwrap_or_else(|| panic!("Campaign not found"));

    // Only allow distribution for successful campaigns
    if campaign.status != CampaignStatus::Successful {
        panic!("Campaign is not successful");
    }

    // Get contributions
    let contributions = contribution::get_contributions(env.clone(), campaign_id.clone());

    // Calculate total rewards to distribute (10% of total funded)
    let total_rewards = campaign.total_funded / 10;

    // Distribute rewards proportionally to contributions
    for contribution in contributions.iter() {
        let reward_amount = (contribution.amount * total_rewards) / campaign.total_funded;
        
        if reward_amount > 0 {
            utils::transfer_tokens(
                &env,
                &env.current_contract_address(),
                &contribution.contributor_id,
                reward_amount,
            );
        }
    }

    // Transfer remaining funds to farmer
    let farmer_amount = campaign.total_funded - total_rewards;
    utils::transfer_tokens(
        &env,
        &env.current_contract_address(),
        &campaign.farmer_id,
        farmer_amount,
    );
}