#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal, Symbol,
};

use crate::{
    campaign::CampaignStatus, CrowdfundingFarmerContract, CrowdfundingFarmerContractClient,
};

// Simple mock token contract for testing
#[contract]
pub struct MockTokenContract;

#[contractimpl]
impl MockTokenContract {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let balance_key = symbol_short!("balance");
        let from_balance: i128 = env
            .storage()
            .persistent()
            .get(&(balance_key.clone(), from.clone()))
            .unwrap_or(0);
        let to_balance: i128 = env
            .storage()
            .persistent()
            .get(&(balance_key.clone(), to.clone()))
            .unwrap_or(0);

        if from_balance < amount {
            panic!("insufficient balance");
        }

        env.storage()
            .persistent()
            .set(&(balance_key.clone(), from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&(balance_key, to), &(to_balance + amount));
    }

    pub fn balance(env: Env, account: Address) -> i128 {
        let balance_key = symbol_short!("balance");
        env.storage()
            .persistent()
            .get(&(balance_key, account))
            .unwrap_or(0)
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let balance_key = symbol_short!("balance");
        let current_balance: i128 = env
            .storage()
            .persistent()
            .get(&(balance_key.clone(), to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&(balance_key, to), &(current_balance + amount));
    }
}

fn create_token_contract(env: &Env, _admin: &Address) -> Address {
    env.register_contract(None, MockTokenContract)
}

// Helper to mint tokens using the mock token contract
fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    env.invoke_contract::<()>(token, &symbol_short!("mint"), (to, &amount).into_val(env));
}

fn set_token_balance(env: &Env, token: &Address, account: &Address, amount: i128) {
    env.as_contract(token, || {
        let balance_key = Symbol::new(env, "balance");
        let supply_key = Symbol::new(env, "supply");

        // Set the account balance
        env.storage()
            .persistent()
            .set(&(balance_key, account), &amount);

        // Also set the total supply to ensure the token has enough supply
        let current_supply: i128 = env.storage().persistent().get(&supply_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&supply_key, &(current_supply + amount));
    });
}

fn setup_successful_campaign(
    env: &Env,
    client: &CrowdfundingFarmerContractClient,
    contract_id: &Address,
) -> (Address, Address, BytesN<32>, [Address; 3]) {
    let farmer = Address::generate(env);
    let reward_token = create_token_contract(env, &farmer);
    let goal_amount = 10000;
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);

    // Create contributors and make contributions
    let contributors = [
        Address::generate(env),
        Address::generate(env),
        Address::generate(env),
    ];
    let amounts = [3000, 4000, 3000]; // Total: 10000 (exactly the goal)

    // Mint tokens for all contributors
    for (contributor, amount) in contributors.iter().zip(amounts.iter()) {
        mint_tokens(env, &reward_token, contributor, *amount);
    }

    // Mock auth for each contributor with token transfer authorization
    for (contributor, amount) in contributors.iter().zip(amounts.iter()) {
        env.mock_auths(&[MockAuth {
            address: contributor,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "contribute",
                args: (contributor.clone(), campaign_id.clone(), *amount).into_val(env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &reward_token,
                    fn_name: "transfer",
                    args: (contributor.clone(), contract_id.clone(), *amount).into_val(env),
                    sub_invokes: &[],
                }],
            },
        }]);
        client.contribute(contributor, &campaign_id, amount);
    }

    // Mark campaign as successful
    let mut campaign = client.get_campaign_details(&campaign_id);
    campaign.status = CampaignStatus::Successful;
    env.as_contract(contract_id, || {
        env.storage().persistent().set(&campaign_id, &campaign);
    });

    (farmer, reward_token, campaign_id, contributors)
}

#[test]
fn test_reward_distribution_successful_campaign() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    // Setup successful campaign
    let (farmer, reward_token, campaign_id, contributors) =
        setup_successful_campaign(&env, &client, &contract_id);

    // Mint some reward tokens to the contract for distribution
    let total_rewards = 1000; // 10% of 10000 goal
    mint_tokens(&env, &reward_token, &contract_id, total_rewards);

    // Distribute rewards
    client.distribute_rewards(&campaign_id);

    // Verify that contributors received proportional rewards
    // Contributor 1: 3000/10000 * 1000 = 300
    let balance1 = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (contributors[0].clone(),).into_val(&env),
    );
    assert_eq!(balance1, 300);

    // Contributor 2: 4000/10000 * 1000 = 400
    let balance2 = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (contributors[1].clone(),).into_val(&env),
    );
    assert_eq!(balance2, 400);

    // Contributor 3: 3000/10000 * 1000 = 300
    let balance3 = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (contributors[2].clone(),).into_val(&env),
    );
    assert_eq!(balance3, 300);

    // Verify farmer received remaining funds (10000 - 1000 = 9000)
    let farmer_balance = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (farmer.clone(),).into_val(&env),
    );
    assert_eq!(farmer_balance, 9000);
}

#[test]
fn test_reward_distribution_proportional_calculation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    // Setup successful campaign with different contribution amounts
    let farmer = Address::generate(&env);
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 10000;
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);

    // Create contributors with different amounts
    let contributors = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];
    let amounts = [1000, 2000, 7000]; // Total: 10000 (exactly the goal)

    // Mint tokens for all contributors
    for (contributor, amount) in contributors.iter().zip(amounts.iter()) {
        mint_tokens(&env, &reward_token, contributor, *amount);
    }

    // Mock auth for each contributor with token transfer authorization
    for (contributor, amount) in contributors.iter().zip(amounts.iter()) {
        env.mock_auths(&[MockAuth {
            address: contributor,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "contribute",
                args: (contributor.clone(), campaign_id.clone(), *amount).into_val(&env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &reward_token,
                    fn_name: "transfer",
                    args: (contributor.clone(), contract_id.clone(), *amount).into_val(&env),
                    sub_invokes: &[],
                }],
            },
        }]);
        client.contribute(contributor, &campaign_id, amount);
    }

    // Mark campaign as successful
    let mut campaign = client.get_campaign_details(&campaign_id);
    campaign.status = CampaignStatus::Successful;
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&campaign_id, &campaign);
    });

    // Mint some reward tokens to the contract for distribution
    let total_rewards = 1000; // 10% of 10000 goal
    mint_tokens(&env, &reward_token, &contract_id, total_rewards);

    // Distribute rewards
    client.distribute_rewards(&campaign_id);

    // Verify proportional distribution
    // Contributor 1: 1000/10000 * 1000 = 100
    let balance1 = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (contributors[0].clone(),).into_val(&env),
    );
    assert_eq!(balance1, 100);

    // Contributor 2: 2000/10000 * 1000 = 200
    let balance2 = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (contributors[1].clone(),).into_val(&env),
    );
    assert_eq!(balance2, 200);

    // Contributor 3: 7000/10000 * 1000 = 700
    let balance3 = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (contributors[2].clone(),).into_val(&env),
    );
    assert_eq!(balance3, 700);

    // Verify farmer received remaining funds (10000 - 1000 = 9000)
    let farmer_balance = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (farmer.clone(),).into_val(&env),
    );
    assert_eq!(farmer_balance, 9000);
}

#[test]
fn test_reward_distribution_zero_contributions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    // Setup successful campaign with no contributions
    let farmer = Address::generate(&env);
    let reward_token = create_token_contract(&env, &farmer);
    let goal_amount = 10000;
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);

    // Mark campaign as successful manually (no contributions made)
    let mut campaign = client.get_campaign_details(&campaign_id);
    campaign.status = CampaignStatus::Successful;
    campaign.total_funded = 0; // No contributions
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&campaign_id, &campaign);
    });

    // Mint some reward tokens to the contract for distribution
    let total_rewards = 1000;
    mint_tokens(&env, &reward_token, &contract_id, total_rewards);

    // Distribute rewards - should not panic even with zero contributions
    client.distribute_rewards(&campaign_id);

    // Verify farmer received all funds (since no contributors)
    let farmer_balance = env.invoke_contract::<i128>(
        &reward_token,
        &symbol_short!("balance"),
        (farmer.clone(),).into_val(&env),
    );
    assert_eq!(farmer_balance, total_rewards);
}
