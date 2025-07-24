#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    symbol_short, Address, BytesN, Env,
};

use crate::{
    claims::{self, Claim},
    insurance::{self, get_policy},
    payouts,
    utils::DataKey,
    FarmerInsuranceContract,
};

#[test]
fn test_full_insurance_flow() {
    let env = Env::default();
    let farmer = Address::generate(&env);
    let admin = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = env.register(FarmerInsuranceContract, ());

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let policy = env.as_contract(&contract_id, || {
        get_policy(env.clone(), policy_id.clone())
    });

    assert_eq!(policy.coverage, symbol_short!("drought"));
    assert_eq!(policy.premium, 100);
    assert!(!policy.active);

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    let updated_policy = env.as_contract(&contract_id, || {
        get_policy(env.clone(), policy_id.clone())
    });

    assert!(updated_policy.active);

    let event_hash = BytesN::random(&env);
    let payout_amount = 300;

    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash.clone(), payout_amount).unwrap()
    });

    let claim = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
            .unwrap()
    });

    assert_eq!(claim.payout_amount, payout_amount);

    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id.clone(), admin.clone())
    });

    let claim_stored = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
    });

    assert!(claim_stored.is_none());
}

#[test]
fn test_create_pol_generates_unique_policy_ids() {
    let env = Env::default();
    let farmer = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = env.register(FarmerInsuranceContract, ());

    let policy_id_1 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let policy_id_2 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("pest"), 150).unwrap()
    });

    assert_ne!(policy_id_1, policy_id_2);
}

#[test]
fn test_new_policy_is_inactive_by_default() {
    let env = Env::default();
    let farmer = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = env.register(FarmerInsuranceContract, ());

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let policy = env.as_contract(&contract_id, || {
        get_policy(env.clone(), policy_id.clone())
    });

    assert!(!policy.active);
}

#[test]
#[should_panic(expected = "Premium already paid")]
fn test_pay_prem_fails_when_called_twice() {
    let env = Env::default();
    let farmer = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = env.register(FarmerInsuranceContract, ());

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });
}

#[test]
#[should_panic(expected = "Policy is not active")]
fn test_sub_claim_fails_if_policy_not_active() {
    let env = Env::default();
    let farmer = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = env.register(FarmerInsuranceContract, ());

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let event_hash = BytesN::random(&env);

    env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash.clone(), 300).unwrap()
    });
}

#[test]
#[should_panic(expected = "Premium must be positive")]
fn test_create_pol_fails_with_zero_premium() {
    let env = Env::default();
    let farmer = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = env.register(FarmerInsuranceContract, ());

    env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 0).unwrap()
    });
}

#[test]
#[should_panic(expected = "Premium must be positive")]
fn test_create_pol_fails_with_negative_premium() {
    let env = Env::default();
    let farmer = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = env.register(FarmerInsuranceContract, ());

    env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), -100).unwrap()
    });
}