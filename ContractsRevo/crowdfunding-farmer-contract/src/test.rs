#[cfg(test)]
mod test {
    use soroban_sdk::{
        testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
        token, Address, BytesN, Env, IntoVal, Symbol, Vec,
    };

    use crate::{
        campaign::{Campaign, CampaignStatus},
        contribution::Contribution,
        CrowdfundingFarmerContract, CrowdfundingFarmerContractClient,
    };

    fn create_token_contract(env: &Env, admin: &Address) -> Address {
        env.register_stellar_asset_contract(admin.clone())
    }

    fn set_token_balance(env: &Env, token: &Address, account: &Address, amount: i128) {
        env.as_contract(token, || {
            let key = Symbol::new(env, "balance");
            env.storage()
                .persistent()
                .set(&(key.clone(), account), &amount);

            // Also set the contract's balance for transfers
            env.storage()
                .persistent()
                .set(&(key, &env.current_contract_address()), &amount);
        });
    }

    #[test]
    fn test_create_and_get_campaign() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
        let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

        let farmer = Address::generate(&env);
        let reward_token = create_token_contract(&env, &farmer);
        let goal_amount = 1000;
        let deadline = env.ledger().timestamp() + 1000;

        let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);
        let campaign = client.get_campaign_details(&campaign_id);

        assert_eq!(campaign.farmer_id, farmer);
        assert_eq!(campaign.goal_amount, goal_amount);
        assert_eq!(campaign.deadline, deadline);
        assert_eq!(campaign.total_funded, 0);
        assert_eq!(campaign.status, CampaignStatus::Active);
        assert_eq!(campaign.reward_token, reward_token);
    }

    #[test]
    #[should_panic(expected = "Campaign deadline has passed")]
    fn test_contribute_to_inactive_campaign() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
        let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

        // Create valid campaign first
        let farmer = Address::generate(&env);
        let reward_token = create_token_contract(&env, &farmer);
        let goal_amount = 1000;
        let deadline = env.ledger().timestamp() + 1000;
        let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);

        // Fast forward past deadline to make it inactive
        env.ledger().set_timestamp(deadline + 1);

        // Try to contribute
        let contributor = Address::generate(&env);
        client.contribute(&contributor, &campaign_id, &500);
    }

    #[test]
    #[should_panic(expected = "Campaign not found")]
    fn test_nonexistent_campaign() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
        let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

        let fake_campaign_id = BytesN::from_array(&env, &[1; 32]);
        client.get_campaign_details(&fake_campaign_id);
    }

    #[test]
    fn test_multiple_campaigns() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
        let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

        // Create first campaign
        let farmer1 = Address::generate(&env);
        let reward_token1 = create_token_contract(&env, &farmer1);
        let goal_amount1 = 1000;
        let deadline1 = env.ledger().timestamp() + 1000;
        let campaign_id1 =
            client.create_campaign(&farmer1, &goal_amount1, &deadline1, &reward_token1);

        // Create second campaign
        let farmer2 = Address::generate(&env);
        let reward_token2 = create_token_contract(&env, &farmer2);
        let goal_amount2 = 2000;
        let deadline2 = env.ledger().timestamp() + 2000;
        let campaign_id2 =
            client.create_campaign(&farmer2, &goal_amount2, &deadline2, &reward_token2);

        // Verify campaigns are separate
        let campaign1 = client.get_campaign_details(&campaign_id1);
        let campaign2 = client.get_campaign_details(&campaign_id2);

        assert_eq!(campaign1.farmer_id, farmer1);
        assert_eq!(campaign2.farmer_id, farmer2);
        assert_ne!(campaign1.campaign_id, campaign2.campaign_id);
    }
}
