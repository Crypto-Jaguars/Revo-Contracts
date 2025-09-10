#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal, Symbol,
};

use crate::{CrowdfundingFarmerContract, CrowdfundingFarmerContractClient};

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

fn setup_campaign(
    env: &Env,
    client: &CrowdfundingFarmerContractClient,
) -> (Address, Address, BytesN<32>) {
    let farmer = Address::generate(env);
    let reward_token = create_token_contract(env, &farmer);
    let goal_amount = 10000;
    let deadline = env.ledger().timestamp() + 1000;
    let campaign_id = client.create_campaign(&farmer, &goal_amount, &deadline, &reward_token);
    (farmer, reward_token, campaign_id)
}

#[test]
fn test_contribute_with_valid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    let (farmer, reward_token, campaign_id) = setup_campaign(&env, &client);
    let contributor = Address::generate(&env);
    let contribution_amount = 1000;

    // Mint tokens to contributor
    mint_tokens(&env, &reward_token, &contributor, contribution_amount);

    // Mock auth for the contributor - both for the contract call and token transfer
    env.mock_auths(&[MockAuth {
        address: &contributor,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "contribute",
            args: (
                contributor.clone(),
                campaign_id.clone(),
                contribution_amount,
            )
                .into_val(&env),
            sub_invokes: &[MockAuthInvoke {
                contract: &reward_token,
                fn_name: "transfer",
                args: (
                    contributor.clone(),
                    contract_id.clone(),
                    contribution_amount,
                )
                    .into_val(&env),
                sub_invokes: &[],
            }],
        },
    }]);

    // Call contribute function directly
    client.contribute(&contributor, &campaign_id, &contribution_amount);

    let campaign = client.get_campaign_details(&campaign_id);
    assert_eq!(campaign.total_funded, contribution_amount);

    let contributions = client.get_contributions(&campaign_id);
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions.get(0).unwrap().contributor_id, contributor);
    assert_eq!(contributions.get(0).unwrap().amount, contribution_amount);
}

#[test]
fn test_contribute_multiple_contributors() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    let (farmer, reward_token, campaign_id) = setup_campaign(&env, &client);
    let contributors = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];
    let amounts = [1000, 2000, 1500];

    // Mint tokens for all contributors
    for (contributor, amount) in contributors.iter().zip(amounts.iter()) {
        mint_tokens(&env, &reward_token, contributor, *amount);
    }

    let mut total_contributed = 0;

    for (i, (contributor, amount)) in contributors.iter().zip(amounts.iter()).enumerate() {
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

        // Call contribute function directly
        client.contribute(contributor, &campaign_id, amount);
        total_contributed += amount;
    }

    let campaign = client.get_campaign_details(&campaign_id);
    assert_eq!(campaign.total_funded, total_contributed);

    let contributions = client.get_contributions(&campaign_id);
    assert_eq!(contributions.len(), 3);

    for (i, contribution) in contributions.iter().enumerate() {
        assert_eq!(contribution.contributor_id, contributors[i]);
        assert_eq!(contribution.amount, amounts[i]);
    }
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_contribute_invalid_amount_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    let (farmer, reward_token, campaign_id) = setup_campaign(&env, &client);
    let contributor = Address::generate(&env);

    client.contribute(&contributor, &campaign_id, &0);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_contribute_invalid_amount_negative() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    let (farmer, reward_token, campaign_id) = setup_campaign(&env, &client);
    let contributor = Address::generate(&env);

    client.contribute(&contributor, &campaign_id, &-1000);
}

#[test]
#[should_panic(expected = "Campaign not found")]
fn test_contribute_to_nonexistent_campaign() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    let contributor = Address::generate(&env);
    let fake_campaign_id = BytesN::from_array(&env, &[1; 32]);

    client.contribute(&contributor, &fake_campaign_id, &1000);
}

#[test]
#[should_panic(expected = "Campaign deadline has passed")]
fn test_contribute_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingFarmerContract);
    let client = CrowdfundingFarmerContractClient::new(&env, &contract_id);

    let (farmer, reward_token, campaign_id) = setup_campaign(&env, &client);
    let contributor = Address::generate(&env);
    let contribution_amount = 1000;

    set_token_balance(&env, &reward_token, &contributor, contribution_amount);

    let campaign = client.get_campaign_details(&campaign_id);
    env.ledger().set_timestamp(campaign.deadline + 1);

    env.mock_auths(&[MockAuth {
        address: &contributor,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "contribute",
            args: (
                contributor.clone(),
                campaign_id.clone(),
                contribution_amount,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.contribute(&contributor, &campaign_id, &contribution_amount);
}
