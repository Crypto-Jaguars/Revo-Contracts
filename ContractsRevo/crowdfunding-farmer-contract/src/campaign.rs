use soroban_sdk::{contracttype, prng, Address, BytesN, Env};

use crate::utils;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Active = 0,
    Successful = 1,
    Failed = 2,
}

#[contracttype]
#[derive(Clone)]
pub struct Campaign {
    pub campaign_id: BytesN<32>,
    pub farmer_id: Address,
    pub goal_amount: i128,
    pub deadline: u64,
    pub total_funded: i128,
    pub status: CampaignStatus,
    pub reward_token: Address,
}

pub fn create_campaign(
    env: Env,
    farmer_id: Address,
    goal_amount: i128,
    deadline: u64,
    reward_token: Address,
) -> BytesN<32> {
    utils::validate_amount(goal_amount);
    utils::validate_deadline(env.ledger().timestamp(), deadline);

    // Generate random bytes for the campaign ID
    let prng = env.prng();
    let mut random_bytes = [0u8; 32];
    prng.fill(&mut random_bytes);
    let campaign_id = BytesN::from_array(&env, &random_bytes);

    let campaign = Campaign {
        campaign_id: campaign_id.clone(),
        farmer_id,
        goal_amount,
        deadline,
        total_funded: 0,
        status: CampaignStatus::Active,
        reward_token,
    };

    utils::save_campaign(&env, &campaign_id, &campaign);
    campaign_id
}

pub fn get_campaign_details(env: Env, campaign_id: BytesN<32>) -> Campaign {
    utils::read_campaign(&env, &campaign_id).unwrap_or_else(|| panic!("Campaign not found"))
}

pub fn update_campaign_status(env: Env, campaign_id: BytesN<32>) {
    let mut campaign = get_campaign_details(env.clone(), campaign_id.clone());
    let current_time = env.ledger().timestamp();

    if campaign.status == CampaignStatus::Active && current_time >= campaign.deadline {
        campaign.status = if campaign.total_funded >= campaign.goal_amount {
            CampaignStatus::Successful
        } else {
            CampaignStatus::Failed
        };
        utils::save_campaign(&env, &campaign_id, &campaign);
    }
}
