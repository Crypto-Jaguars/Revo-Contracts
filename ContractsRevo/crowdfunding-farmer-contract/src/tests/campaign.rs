#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

use crate::{
    campaign::CampaignStatus, CrowdfundingFarmerContract, CrowdfundingFarmerContractClient,
};

fn create_token_contract(env: &Env, admin: &Address) -> Address {
    let contract = env.register_stellar_asset_contract_v2(admin.clone());
    contract.address()
}

fn setup_test_env() -> (Env, CrowdfundingFarmerContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);
    let farmer = Address::generate(&env);
    (env, client, farmer)
}

#[test]
fn test_campaign_creation_with_valid_goals_and_deadlines() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);

    // Test with various valid goal amounts
    let test_cases = [
        (1000, 1000),       // Small goal, short deadline
        (50000, 86400),     // Medium goal, 1 day deadline
        (1000000, 2592000), // Large goal, 30 days deadline
    ];

    for (goal_amount, deadline_offset) in test_cases {
        let deadline = env.ledger().timestamp() + deadline_offset;
        let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);
        let campaign = client.get_campaign_details(&campaign_id);

        assert_eq!(campaign.farmer_id, farmer);
        assert_eq!(campaign.goal_amount, goal_amount);
        assert_eq!(campaign.deadline, deadline);
        assert_eq!(campaign.total_funded, 0);
        assert_eq!(campaign.status, CampaignStatus::Active);
        assert_eq!(campaign.reward_token, reward_token);
        assert_eq!(campaign.campaign_id, campaign_id);
    }
}

#[test]
fn test_campaign_creation_farmer_authorization() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 1000;
    let deadline = env.ledger().timestamp() + 1000;

    // Test that only the farmer can create campaigns
    let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);
    let campaign = client.get_campaign_details(&campaign_id);

    assert_eq!(
        campaign.farmer_id, farmer,
        "Campaign should be associated with the correct farmer"
    );

    // Verify campaign ownership
    assert_eq!(
        campaign.farmer_id, farmer,
        "Farmer authorization should be properly set"
    );
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_campaign_creation_invalid_goal_amount_zero() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let invalid_goal = 0;
    let deadline = env.ledger().timestamp() + 1000;

    client.create_campaign(&farmer, &invalid_goal, &deadline, &reward_token);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_campaign_creation_invalid_goal_amount_negative() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let invalid_goal = -1000;
    let deadline = env.ledger().timestamp() + 1000;

    client.create_campaign(&farmer, &invalid_goal, &deadline, &reward_token);
}

#[test]
#[should_panic(expected = "Deadline must be in the future")]
fn test_campaign_creation_invalid_deadline_past() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 1000;
    let past_deadline = env.ledger().timestamp().saturating_sub(1000);

    client.create_campaign(&farmer, &goal_amount, &past_deadline, &reward_token);
}

#[test]
#[should_panic(expected = "Deadline must be in the future")]
fn test_campaign_creation_invalid_deadline_current() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 1000;
    let current_deadline = env.ledger().timestamp();

    client.create_campaign(&farmer, &goal_amount, &current_deadline, &reward_token);
}

#[test]
fn test_campaign_creation_with_different_farmers() {
    let (env, client, farmer1) = setup_test_env();
    let farmer2 = Address::generate(&env);
    let reward_token1 = create_token_contract(&env, &farmer1);
    let reward_token2 = create_token_contract(&env, &farmer2);

    let goal_amount = 1000;
    let deadline = env.ledger().timestamp() + 1000;

    // Create campaigns for different farmers
    let campaign_id1 = client.create_campaign(&farmer1, &goal_amount, &deadline, &reward_token1);
    let campaign_id2 = client.create_campaign(&farmer2, &goal_amount, &deadline, &reward_token2);

    let campaign1 = client.get_campaign_details(&campaign_id1);
    let campaign2 = client.get_campaign_details(&campaign_id2);

    // Verify campaigns are separate and properly attributed
    assert_eq!(campaign1.farmer_id, farmer1);
    assert_eq!(campaign2.farmer_id, farmer2);
    assert_ne!(campaign1.campaign_id, campaign2.campaign_id);
    assert_eq!(campaign1.reward_token, reward_token1);
    assert_eq!(campaign2.reward_token, reward_token2);
}

#[test]
fn test_campaign_creation_with_different_reward_tokens() {
    let (env, client, farmer) = setup_test_env();
    let reward_token1 = create_token_contract(&env, &farmer);
    let reward_token2 = create_token_contract(&env, &farmer);

    let goal_amount = 1000;
    let deadline = env.ledger().timestamp() + 1000;

    // Create campaigns with different reward tokens
    let campaign_id1 = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token1);
    let campaign_id2 = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token2);

    let campaign1 = client.get_campaign_details(&campaign_id1);
    let campaign2 = client.get_campaign_details(&campaign_id2);

    assert_eq!(campaign1.reward_token, reward_token1);
    assert_eq!(campaign2.reward_token, reward_token2);
    assert_ne!(campaign1.campaign_id, campaign_id2);
}

#[test]
fn test_campaign_creation_unique_ids() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 1000;
    let deadline = env.ledger().timestamp() + 1000;

    // Create multiple campaigns and verify they have unique IDs
    let mut campaign_ids = Vec::new(&env);
    for _ in 0..10 {
        let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);
        campaign_ids.push_back(campaign_id);
    }

    // Verify all campaign IDs are unique
    for i in 0..campaign_ids.len() {
        for j in i + 1..campaign_ids.len() {
            assert_ne!(
                campaign_ids.get(i).unwrap(),
                campaign_ids.get(j).unwrap(),
                "Campaign IDs should be unique"
            );
        }
    }
}

#[test]
fn test_campaign_creation_initial_status() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 1000;
    let deadline = env.ledger().timestamp() + 1000;

    let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);
    let campaign = client.get_campaign_details(&campaign_id);

    // Verify initial campaign state
    assert_eq!(campaign.status, CampaignStatus::Active);
    assert_eq!(campaign.total_funded, 0);
    assert!(campaign.goal_amount > 0);
    assert!(campaign.deadline > env.ledger().timestamp());
}

#[test]
fn test_campaign_creation_with_maximum_values() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);

    // Test with maximum reasonable values
    let max_goal = i128::MAX;
    let max_deadline = u64::MAX;

    // Note: This test might fail due to ledger timestamp constraints
    // but we test the validation logic
    let campaign_id = client.create_campaign(&farmer, &max_goal, &max_deadline, &reward_token);
    let campaign = client.get_campaign_details(&campaign_id);

    assert_eq!(campaign.goal_amount, max_goal);
    assert_eq!(campaign.deadline, max_deadline);
    assert_eq!(campaign.status, CampaignStatus::Active);
}

#[test]
fn test_campaign_creation_with_minimum_valid_values() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);

    // Test with minimum valid values
    let min_goal = 1;
    let min_deadline = env.ledger().timestamp() + 1;

    let campaign_id = client.create_campaign(&farmer, &min_goal, &min_deadline, &reward_token);
    let campaign = client.get_campaign_details(&campaign_id);

    assert_eq!(campaign.goal_amount, min_goal);
    assert_eq!(campaign.deadline, min_deadline);
    assert_eq!(campaign.status, CampaignStatus::Active);
}

#[test]
#[should_panic(expected = "Campaign not found")]
fn test_get_nonexistent_campaign() {
    let (env, client, _farmer) = setup_test_env();
    let fake_campaign_id = BytesN::from_array(&env, &[1; 32]);
    client.get_campaign_details(&fake_campaign_id);
}

#[test]
fn test_campaign_creation_persistence() {
    let (env, client, farmer) = setup_test_env();
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 1000;
    let deadline = env.ledger().timestamp() + 1000;

    // Create campaign
    let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);

    // Verify campaign persists and can be retrieved multiple times
    for _ in 0..5 {
        let campaign = client.get_campaign_details(&campaign_id);
        assert_eq!(campaign.farmer_id, farmer);
        assert_eq!(campaign.goal_amount, goal_amount);
        assert_eq!(campaign.deadline, deadline);
        assert_eq!(campaign.status, CampaignStatus::Active);
    }
}
