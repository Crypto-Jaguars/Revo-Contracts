#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env,
};

use super::utils::{create_test_accounts, create_test_contract};
use crate::insurance::{self, get_policy};

#[test]
fn test_create_pol_generates_unique_policy_ids() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

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
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let policy = env.as_contract(&contract_id, || get_policy(env.clone(), policy_id.clone()));

    assert!(!policy.active);
}

#[test]
#[should_panic(expected = "Premium already paid")]
fn test_pay_prem_fails_when_called_twice() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

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
#[should_panic(expected = "Premium must be positive")]
fn test_create_pol_fails_with_zero_premium() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 0).unwrap()
    });
}

#[test]
#[should_panic(expected = "Premium must be positive")]
fn test_create_pol_fails_with_negative_premium() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), -100).unwrap()
    });
}

#[test]
fn test_farmer_eligibility_multiple_policies() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Farmer should be able to create multiple policies with different coverage types
    let policy_id_1 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let policy_id_2 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("flood"), 200).unwrap()
    });

    let policy_id_3 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("pest"), 150).unwrap()
    });

    // All policies should be created successfully
    let policy_1 = env.as_contract(&contract_id, || {
        get_policy(env.clone(), policy_id_1.clone())
    });
    let policy_2 = env.as_contract(&contract_id, || {
        get_policy(env.clone(), policy_id_2.clone())
    });
    let policy_3 = env.as_contract(&contract_id, || {
        get_policy(env.clone(), policy_id_3.clone())
    });

    assert_eq!(policy_1.coverage, symbol_short!("drought"));
    assert_eq!(policy_2.coverage, symbol_short!("flood"));
    assert_eq!(policy_3.coverage, symbol_short!("pest"));
    assert_eq!(policy_1.farmer, farmer);
    assert_eq!(policy_2.farmer, farmer);
    assert_eq!(policy_3.farmer, farmer);
}

#[test]
fn test_multiple_coverage_types_validation() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Test various coverage types
    let drought_policy = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let flood_policy = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("flood"), 150).unwrap()
    });

    let pest_policy = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("pest"), 120).unwrap()
    });

    let fire_policy = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("fire"), 200).unwrap()
    });

    // Verify each policy has correct coverage type
    let drought = env.as_contract(&contract_id, || get_policy(env.clone(), drought_policy));
    let flood = env.as_contract(&contract_id, || get_policy(env.clone(), flood_policy));
    let pest = env.as_contract(&contract_id, || get_policy(env.clone(), pest_policy));
    let fire = env.as_contract(&contract_id, || get_policy(env.clone(), fire_policy));

    assert_eq!(drought.coverage, symbol_short!("drought"));
    assert_eq!(flood.coverage, symbol_short!("flood"));
    assert_eq!(pest.coverage, symbol_short!("pest"));
    assert_eq!(fire.coverage, symbol_short!("fire"));
}

#[test]
fn test_policy_premium_payment_lifecycle() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 250).unwrap()
    });

    // Policy should start inactive
    let initial_policy =
        env.as_contract(&contract_id, || get_policy(env.clone(), policy_id.clone()));
    assert!(!initial_policy.active);
    assert_eq!(initial_policy.premium, 250);

    // After paying premium, policy should become active
    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    let active_policy =
        env.as_contract(&contract_id, || get_policy(env.clone(), policy_id.clone()));
    assert!(active_policy.active);
    assert_eq!(active_policy.premium, 250);
}

#[test]
fn test_high_volume_policy_creation() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    let mut policy_ids = soroban_sdk::vec!(&env);

    // Create 10 policies with different premiums
    for i in 1..=10 {
        let policy_id = env.as_contract(&contract_id, || {
            insurance::create_pol(
                env.clone(),
                farmer.clone(),
                symbol_short!("drought"),
                100 * i as i128,
            )
            .unwrap()
        });
        policy_ids.push_back((policy_id, 100 * i as i128));
    }

    // Verify all policies were created with correct premiums
    for (policy_id, expected_premium) in policy_ids {
        let policy = env.as_contract(&contract_id, || get_policy(env.clone(), policy_id));
        assert_eq!(policy.farmer, farmer);
        assert_eq!(policy.premium, expected_premium);
        assert_eq!(policy.coverage, symbol_short!("drought"));
        assert!(!policy.active); // Should all start inactive
    }
}

#[test]
fn test_policy_creation_with_different_farmers() {
    let env = Env::default();
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);
    let farmer3 = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    let policy1 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer1.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let policy2 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer2.clone(), symbol_short!("flood"), 200).unwrap()
    });

    let policy3 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer3.clone(), symbol_short!("pest"), 150).unwrap()
    });

    // Verify each farmer owns their respective policy
    let p1 = env.as_contract(&contract_id, || get_policy(env.clone(), policy1.clone()));
    let p2 = env.as_contract(&contract_id, || get_policy(env.clone(), policy2.clone()));
    let p3 = env.as_contract(&contract_id, || get_policy(env.clone(), policy3.clone()));

    assert_eq!(p1.farmer, farmer1);
    assert_eq!(p2.farmer, farmer2);
    assert_eq!(p3.farmer, farmer3);

    assert_ne!(policy1, policy2);
    assert_ne!(policy2, policy3);
    assert_ne!(policy1, policy3);
}

#[test]
#[should_panic(expected = "Policy not found")]
fn test_pay_premium_for_nonexistent_policy() {
    let env = Env::default();
    let (_farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Try to pay premium for non-existent policy
    let fake_policy_id = BytesN::random(&env);

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), fake_policy_id)
    });
}

#[test]
#[should_panic(expected = "Policy not found")]
fn test_get_policy_for_nonexistent_policy() {
    let env = Env::default();
    let (_farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Try to get non-existent policy
    let fake_policy_id = BytesN::random(&env);

    env.as_contract(&contract_id, || get_policy(env.clone(), fake_policy_id));
}

#[test]
fn test_policy_creation_edge_case_maximum_values() {
    let env = Env::default();
    let (farmer, _admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Test with maximum i128 value
    let max_premium = i128::MAX;

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(
            env.clone(),
            farmer.clone(),
            symbol_short!("extreme"),
            max_premium,
        )
        .unwrap()
    });

    let policy = env.as_contract(&contract_id, || get_policy(env.clone(), policy_id));

    assert_eq!(policy.premium, max_premium);
    assert_eq!(policy.farmer, farmer);
    assert_eq!(policy.coverage, symbol_short!("extreme"));
    assert!(!policy.active);
}
