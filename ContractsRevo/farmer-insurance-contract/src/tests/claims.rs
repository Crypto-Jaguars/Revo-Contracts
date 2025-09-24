#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env,
};

use super::utils::{create_test_accounts, create_test_contract};
use crate::{claims, insurance};

#[test]
#[should_panic(expected = "Policy is not active")]
fn test_sub_claim_fails_if_policy_not_active() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let event_hash = BytesN::random(&env);

    env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash.clone(), 300).unwrap()
    });
}

#[test]
fn test_valid_claim_submission_with_correct_event_data() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Create event hash with proper data
    let event_hash = BytesN::random(&env);
    let payout_amount = 500;

    // Submit claim should succeed
    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(
            env.clone(),
            policy_id.clone(),
            event_hash.clone(),
            payout_amount,
        )
        .unwrap()
    });

    // Verify claim was created correctly
    use crate::claims::Claim;
    use crate::utils::DataKey;

    let claim = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
            .unwrap()
    });

    assert_eq!(claim.policy_id, policy_id);
    assert_eq!(claim.event_hash, event_hash);
    assert_eq!(claim.payout_amount, payout_amount);
}

#[test]
fn test_multiple_claims_for_same_policy() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Submit multiple claims for the same policy
    let event_hash_1 = BytesN::random(&env);
    let event_hash_2 = BytesN::random(&env);
    let event_hash_3 = BytesN::random(&env);

    let claim_id_1 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash_1.clone(), 200).unwrap()
    });

    let claim_id_2 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash_2.clone(), 300).unwrap()
    });

    let claim_id_3 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash_3.clone(), 400).unwrap()
    });

    // Verify all claims are different and correctly stored
    use crate::claims::Claim;
    use crate::utils::DataKey;

    let claim_1 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id_1.clone()))
            .unwrap()
    });
    let claim_2 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id_2.clone()))
            .unwrap()
    });
    let claim_3 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id_3.clone()))
            .unwrap()
    });

    assert_ne!(claim_id_1, claim_id_2);
    assert_ne!(claim_id_2, claim_id_3);
    assert_ne!(claim_id_1, claim_id_3);

    assert_eq!(claim_1.policy_id, policy_id);
    assert_eq!(claim_2.policy_id, policy_id);
    assert_eq!(claim_3.policy_id, policy_id);

    assert_eq!(claim_1.payout_amount, 200);
    assert_eq!(claim_2.payout_amount, 300);
    assert_eq!(claim_3.payout_amount, 400);
}

#[test]
fn test_claim_with_zero_payout_amount() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    let event_hash = BytesN::random(&env);

    // Submit claim with zero payout - should succeed (business logic allows it)
    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash.clone(), 0).unwrap()
    });

    use crate::claims::Claim;
    use crate::utils::DataKey;

    let claim = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
            .unwrap()
    });

    assert_eq!(claim.payout_amount, 0);
}

#[test]
fn test_claim_with_negative_payout_amount() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    let event_hash = BytesN::random(&env);

    // Submit claim with negative payout - should succeed (no validation in current contract)
    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash.clone(), -100).unwrap()
    });

    use crate::claims::Claim;
    use crate::utils::DataKey;

    let claim = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
            .unwrap()
    });

    assert_eq!(claim.payout_amount, -100);
}

#[test]
#[should_panic(expected = "Policy not found")]
fn test_claim_submission_with_invalid_policy_id() {
    let env = Env::default();
    let (_farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Use a random policy ID that doesn't exist
    let invalid_policy_id = BytesN::random(&env);
    let event_hash = BytesN::random(&env);

    env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), invalid_policy_id, event_hash, 300).unwrap()
    });
}

#[test]
fn test_event_hash_verification_different_events() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Test different event hashes for different types of events
    let drought_event = BytesN::random(&env);
    let flood_event = BytesN::random(&env);
    let temperature_event = BytesN::random(&env);

    let claim_1 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), drought_event.clone(), 500).unwrap()
    });

    let claim_2 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), flood_event.clone(), 300).unwrap()
    });

    let claim_3 = env.as_contract(&contract_id, || {
        claims::sub_claim(
            env.clone(),
            policy_id.clone(),
            temperature_event.clone(),
            200,
        )
        .unwrap()
    });

    use crate::claims::Claim;
    use crate::utils::DataKey;

    let stored_claim_1 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_1))
            .unwrap()
    });
    let stored_claim_2 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_2))
            .unwrap()
    });
    let stored_claim_3 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_3))
            .unwrap()
    });

    assert_eq!(stored_claim_1.event_hash, drought_event);
    assert_eq!(stored_claim_2.event_hash, flood_event);
    assert_eq!(stored_claim_3.event_hash, temperature_event);

    assert_ne!(stored_claim_1.event_hash, stored_claim_2.event_hash);
    assert_ne!(stored_claim_2.event_hash, stored_claim_3.event_hash);
}

#[test]
fn test_claim_submission_high_volume() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("disaster"), 1000).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Submit 15 claims
    let mut claim_ids = soroban_sdk::vec!(&env);
    for i in 1..=15 {
        let event_hash = BytesN::random(&env);
        let payout_amount = i * 100;

        let claim_id = env.as_contract(&contract_id, || {
            claims::sub_claim(
                env.clone(),
                policy_id.clone(),
                event_hash.clone(),
                payout_amount,
            )
            .unwrap()
        });

        claim_ids.push_back((claim_id, payout_amount, event_hash));
    }

    // Verify all claims were created correctly
    use crate::claims::Claim;
    use crate::utils::DataKey;

    for (claim_id, expected_amount, expected_event) in claim_ids {
        let claim = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
                .unwrap()
        });

        assert_eq!(claim.policy_id, policy_id);
        assert_eq!(claim.payout_amount, expected_amount);
        assert_eq!(claim.event_hash, expected_event);
    }
}

#[test]
fn test_claim_with_maximum_payout_amount() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("maxclaim"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    let event_hash = BytesN::random(&env);
    let max_payout = i128::MAX;

    // Submit claim with maximum payout amount
    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(
            env.clone(),
            policy_id.clone(),
            event_hash.clone(),
            max_payout,
        )
        .unwrap()
    });

    use crate::claims::Claim;
    use crate::utils::DataKey;

    let claim = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
            .unwrap()
    });

    assert_eq!(claim.payout_amount, max_payout);
    assert_eq!(claim.event_hash, event_hash);
}

#[test]
fn test_claim_integration_across_multiple_policies() {
    let env = Env::default();
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);
    let farmer3 = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate multiple policies for different farmers
    let policy1 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer1.clone(), symbol_short!("drought"), 100).unwrap()
    });
    let policy2 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer2.clone(), symbol_short!("flood"), 200).unwrap()
    });
    let policy3 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer3.clone(), symbol_short!("fire"), 300).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy1.clone())
    });
    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy2.clone())
    });
    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy3.clone())
    });

    // Submit claims for each policy
    let event1 = BytesN::random(&env);
    let event2 = BytesN::random(&env);
    let event3 = BytesN::random(&env);

    let claim1 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy1.clone(), event1.clone(), 500).unwrap()
    });
    let claim2 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy2.clone(), event2.clone(), 750).unwrap()
    });
    let claim3 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy3.clone(), event3.clone(), 1000).unwrap()
    });

    // Verify all claims are isolated and correct
    use crate::claims::Claim;
    use crate::utils::DataKey;

    let stored_claim1 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim1))
            .unwrap()
    });
    let stored_claim2 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim2))
            .unwrap()
    });
    let stored_claim3 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim3))
            .unwrap()
    });

    assert_eq!(stored_claim1.policy_id, policy1);
    assert_eq!(stored_claim2.policy_id, policy2);
    assert_eq!(stored_claim3.policy_id, policy3);

    assert_eq!(stored_claim1.payout_amount, 500);
    assert_eq!(stored_claim2.payout_amount, 750);
    assert_eq!(stored_claim3.payout_amount, 1000);
}
