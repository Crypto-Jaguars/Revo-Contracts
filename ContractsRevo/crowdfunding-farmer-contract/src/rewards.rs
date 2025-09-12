use soroban_sdk::{symbol_short, Address, BytesN, Env, IntoVal};

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

    let contributions = contribution::get_contributions(env.clone(), campaign_id.clone());
    let total_rewards = campaign.total_funded / 10;

    // Distribute rewards to contributors using mock token contract
    if campaign.total_funded > 0 {
        for contribution in contributions.iter() {
            let reward_amount = (contribution.amount * total_rewards) / campaign.total_funded;
            if reward_amount > 0 {
                env.invoke_contract::<()>(
                    &campaign.reward_token,
                    &symbol_short!("transfer"),
                    (
                        env.current_contract_address(),
                        contribution.contributor_id.clone(),
                        reward_amount,
                    )
                        .into_val(&env),
                );
            }
        }

        // Give remaining funds to farmer
        let farmer_amount = campaign.total_funded - total_rewards;
        if farmer_amount > 0 {
            env.invoke_contract::<()>(
                &campaign.reward_token,
                &symbol_short!("transfer"),
                (
                    env.current_contract_address(),
                    campaign.farmer_id,
                    farmer_amount,
                )
                    .into_val(&env),
            );
        }
    } else {
        // If no contributions were made, give all available tokens to farmer
        // This handles the case where the contract has tokens but no contributions
        let contract_balance = env.invoke_contract::<i128>(
            &campaign.reward_token,
            &symbol_short!("balance"),
            (env.current_contract_address(),).into_val(&env),
        );

        if contract_balance > 0 {
            env.invoke_contract::<()>(
                &campaign.reward_token,
                &symbol_short!("transfer"),
                (
                    env.current_contract_address(),
                    campaign.farmer_id,
                    contract_balance,
                )
                    .into_val(&env),
            );
        }
    }
}
