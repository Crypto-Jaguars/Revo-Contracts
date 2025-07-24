use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use crate::{CampaignStatus, CrowdfundingFarmerContract, CrowdfundingFarmerContractClient};

#[test]
fn test_create_and_fund_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    // Register contract using new method
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    // Setup test accounts - use Address::generate() instead of random()
    let farmer = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let reward_token = Address::generate(&env);

    // Create campaign
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &1000, &deadline, &reward_token);

    // Contribute to campaign
    client.contribute(&contributor1, &campaign_id, &400);
    client.contribute(&contributor2, &campaign_id, &600);

    // Check campaign details
    let campaign = client.get_campaign_details(&campaign_id);
    assert_eq!(campaign.total_funded, 1000);
    assert_eq!(campaign.status, CampaignStatus::Active);

    // Fast forward past deadline
    env.ledger().with_mut(|l| l.timestamp = deadline + 1);

    // Distribute rewards
    client.distribute_rewards(&campaign_id);

    // Check campaign is now successful
    let campaign = client.get_campaign_details(&campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Successful);
}

#[test]
fn test_failed_campaign_refund() {
    let env = Env::default();
    env.mock_all_auths();

    // Register contract using new method
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    // Setup test accounts - use Address::generate() instead of random()
    let farmer = Address::generate(&env);
    let contributor = Address::generate(&env);
    let reward_token = Address::generate(&env);

    // Create campaign
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &1000, &deadline, &reward_token);

    // Contribute to campaign (but not enough to reach goal)
    client.contribute(&contributor, &campaign_id, &400);

    // Fast forward past deadline
    env.ledger().with_mut(|l| l.timestamp = deadline + 1);

    // Refund contributions
    client.refund_contributions(&campaign_id);

    // Check campaign is now failed
    let campaign = client.get_campaign_details(&campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Failed);
}
