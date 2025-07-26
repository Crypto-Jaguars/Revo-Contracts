use soroban_sdk::{contracttype, token, Address, BytesN, Env, Vec};

use crate::{utils, CampaignStatus};

#[contracttype]
#[derive(Clone)]
pub struct Contribution {
    pub contributor_id: Address,
    pub campaign_id: BytesN<32>,
    pub amount: i128,
}

pub fn contribute(env: Env, contributor: Address, campaign_id: BytesN<32>, amount: i128) {
    utils::validate_amount(amount);

    let mut campaign =
        utils::read_campaign(&env, &campaign_id).unwrap_or_else(|| panic!("Campaign not found"));

    if campaign.status != CampaignStatus::Active {
        panic!("Campaign is not active");
    }

    if env.ledger().timestamp() >= campaign.deadline {
        panic!("Campaign deadline has passed");
    }

    // Require auth from contributor
    contributor.require_auth();

    // Transfer tokens from contributor to contract
    let token_client = token::Client::new(&env, &campaign.reward_token);
    token_client.transfer(&contributor, &env.current_contract_address(), &amount);

    campaign.total_funded += amount;
    utils::save_campaign(&env, &campaign_id, &campaign);

    let mut contributions =
        utils::read_contributions(&env, &campaign_id).unwrap_or_else(|| Vec::new(&env));
    contributions.push_back(Contribution {
        contributor_id: contributor.clone(),
        campaign_id: campaign_id.clone(),
        amount,
    });
    utils::save_contributions(&env, &campaign_id, &contributions);
}

pub fn refund_contributions(env: Env, campaign_id: BytesN<32>) {
    let campaign =
        utils::read_campaign(&env, &campaign_id).unwrap_or_else(|| panic!("Campaign not found"));

    if campaign.status != CampaignStatus::Failed {
        panic!("Campaign is not failed");
    }

    let contributions = utils::read_contributions(&env, &campaign_id)
        .unwrap_or_else(|| panic!("No contributions found"));

    let token_client = token::Client::new(&env, &campaign.reward_token);
    
    for contribution in contributions.iter() {
        // Require auth from contract (since it's initiating the refund)
        env.current_contract_address().require_auth();
        token_client.transfer(
            &env.current_contract_address(),
            &contribution.contributor_id,
            &contribution.amount,
        );
    }

    utils::save_contributions(&env, &campaign_id, &Vec::new(&env));
}

pub fn get_contributions(env: Env, campaign_id: BytesN<32>) -> Vec<Contribution> {
    utils::read_contributions(&env, &campaign_id).unwrap_or_else(|| Vec::new(&env))
}
