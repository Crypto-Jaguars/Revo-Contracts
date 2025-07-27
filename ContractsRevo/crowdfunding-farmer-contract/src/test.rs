use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::{self, StellarAssetClient}, 
    Address, BytesN, Env, IntoVal,
};

use crate::{CampaignStatus, CrowdfundingFarmerContract, CrowdfundingFarmerContractClient};

#[test]
fn test_create_and_fund_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    // Create and setup mock token
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = StellarAssetClient::new(&env, &token_address.address());

    // Register contract
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    // Setup test accounts
    let farmer = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);

    // Mint tokens to contributors for testing
    token.mock_all_auths();
    token.mint(&contributor1, &1000);
    token.mint(&contributor2, &1000);

    // Create campaign
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &1000, &deadline, &token_address.address());

    // Contribute to campaign - need to mock auth for each contributor
    env.as_contract(&contract_id, || {
        token.mock_all_auths();
        client.contribute(&contributor1, &campaign_id, &400);
    });
    
    env.as_contract(&contract_id, || {
        token.mock_all_auths();
        client.contribute(&contributor2, &campaign_id, &600);
    });

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

    // Create and setup mock token
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = StellarAssetClient::new(&env, &token_address.address());

    // Register contract
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    // Setup test accounts
    let farmer = Address::generate(&env);
    let contributor = Address::generate(&env);

    // Mint tokens to contributor for testing
    token.mock_all_auths();
    token.mint(&contributor, &1000);

    // Create campaign
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &1000, &deadline, &token_address.address());

    // Contribute to campaign
    env.as_contract(&contract_id, || {
        token.mock_all_auths();
        client.contribute(&contributor, &campaign_id, &400);
    });

    // Fast forward past deadline
    env.ledger().with_mut(|l| l.timestamp = deadline + 1);

    // Update campaign status first
    client.get_campaign_details(&campaign_id); 

    // Refund contributions
    client.refund_contributions(&campaign_id);

    // Check campaign is now failed
    let campaign = client.get_campaign_details(&campaign_id);
    assert_eq!(campaign.status, CampaignStatus::Failed);
}