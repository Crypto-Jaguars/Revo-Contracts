use soroban_sdk::{token, Address, BytesN, Env};

use crate::{contribution, utils, CampaignStatus};

#[derive(Clone)]
pub struct Reward {
    pub contributor_id: Address,
    pub campaign_id: BytesN<32>,
    pub amount: i128,
}

pub fn distribute_rewards(env: Env, campaign_id: BytesN<32>) {
    let campaign =
        utils::read_campaign(&env, &campaign_id).unwrap_or_else(|| panic!("Campaign not found"));

    if campaign.status != CampaignStatus::Successful {
        panic!("Campaign is not successful");
    }

    // Update campaign status first
    utils::save_campaign(&env, &campaign_id, &campaign);

    let contributions = contribution::get_contributions(env.clone(), campaign_id.clone());
    let total_rewards = campaign.total_funded / 10;
    let token_client = token::Client::new(&env, &campaign.reward_token);

    // Require auth from contract for reward distribution
    env.current_contract_address().require_auth();

    // Ensure contract has enough balance
    for contribution in contributions.iter() {
        let reward_amount = (contribution.amount * total_rewards) / campaign.total_funded;
        if reward_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &contribution.contributor_id,
                &reward_amount,
            );
        }
    }

    let farmer_amount = campaign.total_funded - total_rewards;
    if farmer_amount > 0 {
        token_client.transfer(
            &env.current_contract_address(),
            &campaign.farmer_id,
            &farmer_amount,
        );
    }
}
