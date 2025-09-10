#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

use crate::{
    campaign::{Campaign, CampaignStatus},
    contribution::Contribution,
    utils, CrowdfundingFarmerContract,
};

fn with_contract_context<F, R>(f: F) -> R
where
    F: FnOnce(&Env) -> R,
{
    let env = Env::default();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    env.as_contract(&contract_id, || f(&env))
}

#[test]
fn test_validate_amount_positive() {
    // Test valid positive amounts
    let valid_amounts = [1, 100, 1000, 1000000, i128::MAX];

    for amount in valid_amounts {
        // Should not panic
        utils::validate_amount(amount);
    }
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_validate_amount_zero() {
    utils::validate_amount(0);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_validate_amount_negative() {
    let negative_amounts = [-1, -100, -1000, i128::MIN];

    for amount in negative_amounts {
        utils::validate_amount(amount);
    }
}

#[test]
fn test_validate_deadline_future() {
    let env = Env::default();
    let current_time = env.ledger().timestamp();
    let future_deadlines = [
        current_time + 1,
        current_time + 1000,
        current_time + 86400,   // 1 day
        current_time + 2592000, // 30 days
        u64::MAX,
    ];

    for deadline in future_deadlines {
        // Should not panic
        utils::validate_deadline(current_time, deadline);
    }
}

#[test]
#[should_panic(expected = "Deadline must be in the future")]
fn test_validate_deadline_past() {
    let env = Env::default();
    let current_time = env.ledger().timestamp();
    let past_deadlines = [
        current_time.saturating_sub(1),
        current_time.saturating_sub(1000),
        0,
    ];

    for deadline in past_deadlines {
        utils::validate_deadline(current_time, deadline);
    }
}

#[test]
#[should_panic(expected = "Deadline must be in the future")]
fn test_validate_deadline_current() {
    let env = Env::default();
    let current_time = env.ledger().timestamp();

    // Should panic for current time
    utils::validate_deadline(current_time, current_time);
}

#[test]
fn test_save_and_read_campaign() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);
        let farmer_id = Address::generate(env);
        let reward_token = Address::generate(env);

        let campaign = Campaign {
            campaign_id: campaign_id.clone(),
            farmer_id: farmer_id.clone(),
            goal_amount: 10000,
            deadline: env.ledger().timestamp() + 1000,
            total_funded: 0,
            status: CampaignStatus::Active,
            reward_token: reward_token.clone(),
        };

        // Save campaign
        utils::save_campaign(env, &campaign_id, &campaign);

        // Read campaign
        let read_campaign = utils::read_campaign(env, &campaign_id).unwrap();

        assert_eq!(read_campaign.campaign_id, campaign.campaign_id);
        assert_eq!(read_campaign.farmer_id, campaign.farmer_id);
        assert_eq!(read_campaign.goal_amount, campaign.goal_amount);
        assert_eq!(read_campaign.deadline, campaign.deadline);
        assert_eq!(read_campaign.total_funded, campaign.total_funded);
        assert_eq!(read_campaign.status, campaign.status);
        assert_eq!(read_campaign.reward_token, campaign.reward_token);
    });
}

#[test]
fn test_read_nonexistent_campaign() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);

        let result = utils::read_campaign(env, &campaign_id);
        assert!(
            result.is_none(),
            "Should return None for nonexistent campaign"
        );
    });
}

#[test]
fn test_save_and_read_contributions() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);

        let mut contributions = Vec::new(env);
        let contribution1 = Contribution {
            contributor_id: Address::generate(env),
            campaign_id: campaign_id.clone(),
            amount: 1000,
        };
        let contribution2 = Contribution {
            contributor_id: Address::generate(env),
            campaign_id: campaign_id.clone(),
            amount: 2000,
        };

        contributions.push_back(contribution1.clone());
        contributions.push_back(contribution2.clone());

        // Save contributions
        utils::save_contributions(env, &campaign_id, &contributions);

        // Read contributions
        let read_contributions = utils::read_contributions(env, &campaign_id).unwrap();

        assert_eq!(read_contributions.len(), 2);
        assert_eq!(
            read_contributions.get(0).unwrap().contributor_id,
            contribution1.contributor_id
        );
        assert_eq!(
            read_contributions.get(0).unwrap().amount,
            contribution1.amount
        );
        assert_eq!(
            read_contributions.get(1).unwrap().contributor_id,
            contribution2.contributor_id
        );
        assert_eq!(
            read_contributions.get(1).unwrap().amount,
            contribution2.amount
        );
    });
}

#[test]
fn test_read_nonexistent_contributions() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);

        let result = utils::read_contributions(env, &campaign_id);
        assert!(
            result.is_none(),
            "Should return None for nonexistent contributions"
        );
    });
}

#[test]
fn test_campaign_persistence_across_calls() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);
        let farmer_id = Address::generate(env);
        let reward_token = Address::generate(env);

        let campaign = Campaign {
            campaign_id: campaign_id.clone(),
            farmer_id: farmer_id.clone(),
            goal_amount: 10000,
            deadline: env.ledger().timestamp() + 1000,
            total_funded: 0,
            status: CampaignStatus::Active,
            reward_token: reward_token.clone(),
        };

        // Save campaign
        utils::save_campaign(env, &campaign_id, &campaign);

        // Read multiple times to ensure persistence
        for _ in 0..5 {
            let read_campaign = utils::read_campaign(env, &campaign_id).unwrap();
            assert_eq!(read_campaign.campaign_id, campaign.campaign_id);
            assert_eq!(read_campaign.farmer_id, campaign.farmer_id);
            assert_eq!(read_campaign.goal_amount, campaign.goal_amount);
        }
    });
}

#[test]
fn test_contributions_persistence_across_calls() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);

        let mut contributions = Vec::new(env);
        let contribution = Contribution {
            contributor_id: Address::generate(env),
            campaign_id: campaign_id.clone(),
            amount: 1000,
        };

        contributions.push_back(contribution.clone());

        // Save contributions
        utils::save_contributions(env, &campaign_id, &contributions);

        // Read multiple times to ensure persistence
        for _ in 0..5 {
            let read_contributions = utils::read_contributions(env, &campaign_id).unwrap();
            assert_eq!(read_contributions.len(), 1);
            assert_eq!(
                read_contributions.get(0).unwrap().contributor_id,
                contribution.contributor_id
            );
            assert_eq!(
                read_contributions.get(0).unwrap().amount,
                contribution.amount
            );
        }
    });
}

#[test]
fn test_campaign_overwrite() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);
        let farmer_id = Address::generate(env);
        let reward_token = Address::generate(env);

        // Create initial campaign
        let initial_campaign = Campaign {
            campaign_id: campaign_id.clone(),
            farmer_id: farmer_id.clone(),
            goal_amount: 10000,
            deadline: env.ledger().timestamp() + 1000,
            total_funded: 0,
            status: CampaignStatus::Active,
            reward_token: reward_token.clone(),
        };

        utils::save_campaign(env, &campaign_id, &initial_campaign);

        // Create updated campaign
        let updated_campaign = Campaign {
            campaign_id: campaign_id.clone(),
            farmer_id: farmer_id.clone(),
            goal_amount: 20000,                        // Changed
            deadline: env.ledger().timestamp() + 2000, // Changed
            total_funded: 5000,                        // Changed
            status: CampaignStatus::Successful,        // Changed
            reward_token: reward_token.clone(),
        };

        utils::save_campaign(env, &campaign_id, &updated_campaign);

        // Verify the campaign was overwritten
        let read_campaign = utils::read_campaign(env, &campaign_id).unwrap();
        assert_eq!(read_campaign.goal_amount, 20000);
        assert_eq!(read_campaign.total_funded, 5000);
        assert_eq!(read_campaign.status, CampaignStatus::Successful);
    });
}

#[test]
fn test_contributions_overwrite() {
    with_contract_context(|env| {
        let campaign_id = BytesN::from_array(env, &[1; 32]);

        // Create initial contributions
        let mut initial_contributions = Vec::new(env);
        let contribution1 = Contribution {
            contributor_id: Address::generate(env),
            campaign_id: campaign_id.clone(),
            amount: 1000,
        };
        initial_contributions.push_back(contribution1);

        utils::save_contributions(env, &campaign_id, &initial_contributions);

        // Create updated contributions
        let mut updated_contributions = Vec::new(env);
        let contribution2 = Contribution {
            contributor_id: Address::generate(env),
            campaign_id: campaign_id.clone(),
            amount: 2000,
        };
        updated_contributions.push_back(contribution2.clone());

        utils::save_contributions(env, &campaign_id, &updated_contributions);

        // Verify the contributions were overwritten
        let read_contributions = utils::read_contributions(env, &campaign_id).unwrap();
        assert_eq!(read_contributions.len(), 1);
        assert_eq!(
            read_contributions.get(0).unwrap().contributor_id,
            contribution2.contributor_id
        );
        assert_eq!(
            read_contributions.get(0).unwrap().amount,
            contribution2.amount
        );
    });
}
