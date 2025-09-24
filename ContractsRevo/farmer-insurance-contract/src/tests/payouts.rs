#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env,
};

use super::utils::{create_test_accounts, create_test_contract};
use crate::{
    claims::{self, Claim},
    insurance::{self, get_policy},
    payouts,
    utils::DataKey,
};

#[test]
fn test_full_insurance_flow() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    let policy = env.as_contract(&contract_id, || get_policy(env.clone(), policy_id.clone()));

    assert_eq!(policy.coverage, symbol_short!("drought"));
    assert_eq!(policy.premium, 100);
    assert!(!policy.active);

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    let updated_policy =
        env.as_contract(&contract_id, || get_policy(env.clone(), policy_id.clone()));

    assert!(updated_policy.active);

    let event_hash = BytesN::random(&env);
    let payout_amount = 300;

    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(
            env.clone(),
            policy_id.clone(),
            event_hash.clone(),
            payout_amount,
        )
        .unwrap()
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
fn test_payout_amount_accuracy_verification() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 500).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Submit multiple claims with different payout amounts
    let event_hash_1 = BytesN::random(&env);
    let event_hash_2 = BytesN::random(&env);
    let event_hash_3 = BytesN::random(&env);

    let claim_id_1 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash_1, 1000).unwrap()
    });

    let claim_id_2 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash_2, 2500).unwrap()
    });

    let claim_id_3 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash_3, 750).unwrap()
    });

    // Verify claims before payout
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

    assert_eq!(claim_1.payout_amount, 1000);
    assert_eq!(claim_2.payout_amount, 2500);
    assert_eq!(claim_3.payout_amount, 750);

    // Process payouts and verify accuracy
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id_1.clone(), admin.clone())
    });

    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id_2.clone(), admin.clone())
    });

    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id_3.clone(), admin.clone())
    });

    // Verify all claims are removed after payout
    let claim_1_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id_1))
    });
    let claim_2_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id_2))
    });
    let claim_3_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id_3))
    });

    assert!(claim_1_after.is_none());
    assert!(claim_2_after.is_none());
    assert!(claim_3_after.is_none());
}

#[test]
#[should_panic(expected = "Policy is not active")]
fn test_payout_for_inactive_policy() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create policy but don't pay premium (stays inactive)
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    // Try to submit claim on inactive policy first
    let event_hash = BytesN::random(&env);

    // This should fail because policy is not active
    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash, 300).unwrap()
    });

    // Try to process payout - should fail because policy is inactive
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id, admin.clone())
    });
}

#[test]
#[should_panic(expected = "Claim not found")]
fn test_payout_for_nonexistent_claim() {
    let env = Env::default();
    let (_farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Try to process payout for a non-existent claim
    let fake_claim_id = BytesN::random(&env);

    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), fake_claim_id, admin.clone())
    });
}

#[test]
fn test_admin_authorization_for_payouts() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);
    let unauthorized_user = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Submit claim
    let event_hash = BytesN::random(&env);
    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash, 300).unwrap()
    });

    // Test that both admin and unauthorized user can process payouts
    // (current contract doesn't restrict admin - any address can be admin)
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id.clone(), admin.clone())
    });

    // Verify claim is removed
    let claim_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
    });

    assert!(claim_after.is_none());

    // Create another claim to test with unauthorized user
    let event_hash_2 = BytesN::random(&env);
    let claim_id_2 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash_2, 400).unwrap()
    });

    // Unauthorized user can also process payouts (no admin restriction in current contract)
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id_2.clone(), unauthorized_user.clone())
    });

    let claim_2_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id_2))
    });

    assert!(claim_2_after.is_none());
}

#[test]
fn test_multiple_payouts_same_policy_different_claims() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("drought"), 200).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Create multiple claims for the same policy
    let mut claim_ids = soroban_sdk::vec!(&env);
    let payout_amounts = soroban_sdk::vec!(&env, 100, 250, 500, 150, 300);

    for i in 0..payout_amounts.len() {
        let amount = payout_amounts.get(i).unwrap();
        let event_hash = BytesN::random(&env);
        let claim_id = env.as_contract(&contract_id, || {
            claims::sub_claim(env.clone(), policy_id.clone(), event_hash, amount).unwrap()
        });
        claim_ids.push_back((claim_id, amount));
    }

    // Process all payouts
    for (claim_id, expected_amount) in claim_ids {
        // Verify claim exists before payout
        let claim = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
                .unwrap()
        });
        assert_eq!(claim.payout_amount, expected_amount);

        // Process payout
        env.as_contract(&contract_id, || {
            payouts::pay_out(env.clone(), claim_id.clone(), admin.clone())
        });

        // Verify claim is removed after payout
        let claim_after = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, Claim>(&DataKey::Claim(claim_id))
        });
        assert!(claim_after.is_none());
    }
}

#[test]
fn test_payout_transparency_and_auditability() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("flood"), 150).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Submit claim
    let event_hash = BytesN::random(&env);
    let payout_amount = 750;

    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(
            env.clone(),
            policy_id.clone(),
            event_hash.clone(),
            payout_amount,
        )
        .unwrap()
    });

    // Verify claim details before payout for audit trail
    let claim_before = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
            .unwrap()
    });

    assert_eq!(claim_before.policy_id, policy_id);
    assert_eq!(claim_before.event_hash, event_hash);
    assert_eq!(claim_before.payout_amount, payout_amount);

    // Process payout
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id.clone(), admin.clone())
    });

    // Verify payout completed (claim removed)
    let claim_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
    });

    assert!(claim_after.is_none());

    // Verify policy is still active after payout
    let policy_after_payout = env.as_contract(&contract_id, || get_policy(env.clone(), policy_id));

    assert!(policy_after_payout.active);
    assert_eq!(policy_after_payout.farmer, farmer);
    assert_eq!(policy_after_payout.coverage, symbol_short!("flood"));
}

#[test]
fn test_high_volume_payout_processing() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("disaster"), 1000).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Create 20 claims
    let mut claim_ids = soroban_sdk::vec!(&env);
    for i in 1..=20 {
        let event_hash = BytesN::random(&env);
        let payout_amount = i * 50;

        let claim_id = env.as_contract(&contract_id, || {
            claims::sub_claim(env.clone(), policy_id.clone(), event_hash, payout_amount).unwrap()
        });

        claim_ids.push_back((claim_id, payout_amount));
    }

    // Process all payouts
    for (claim_id, expected_amount) in claim_ids {
        // Verify claim before payout
        let claim = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
                .unwrap()
        });
        assert_eq!(claim.payout_amount, expected_amount);

        // Process payout
        env.as_contract(&contract_id, || {
            payouts::pay_out(env.clone(), claim_id.clone(), admin.clone())
        });

        // Verify claim removed
        let claim_after = env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<_, Claim>(&DataKey::Claim(claim_id))
        });
        assert!(claim_after.is_none());
    }
}

#[test]
fn test_payout_with_maximum_amount() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate policy
    let policy_id = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer.clone(), symbol_short!("maxpayout"), 100).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy_id.clone())
    });

    // Submit claim with maximum payout
    let event_hash = BytesN::random(&env);
    let max_payout = i128::MAX;

    let claim_id = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy_id.clone(), event_hash, max_payout).unwrap()
    });

    // Verify claim before payout
    let claim_before = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
            .unwrap()
    });
    assert_eq!(claim_before.payout_amount, max_payout);

    // Process payout
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim_id.clone(), admin.clone())
    });

    // Verify payout completed
    let claim_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim_id))
    });
    assert!(claim_after.is_none());
}

#[test]
fn test_cross_policy_payout_integration() {
    let env = Env::default();
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Create and activate multiple policies
    let policy1 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer1.clone(), symbol_short!("drought"), 200).unwrap()
    });
    let policy2 = env.as_contract(&contract_id, || {
        insurance::create_pol(env.clone(), farmer2.clone(), symbol_short!("flood"), 300).unwrap()
    });

    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy1.clone())
    });
    env.as_contract(&contract_id, || {
        insurance::pay_prem(env.clone(), policy2.clone())
    });

    // Submit claims for both policies
    let event1 = BytesN::random(&env);
    let event2 = BytesN::random(&env);

    let claim1 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy1.clone(), event1, 800).unwrap()
    });
    let claim2 = env.as_contract(&contract_id, || {
        claims::sub_claim(env.clone(), policy2.clone(), event2, 1200).unwrap()
    });

    // Process payouts with different admins
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim1.clone(), admin1.clone())
    });
    env.as_contract(&contract_id, || {
        payouts::pay_out(env.clone(), claim2.clone(), admin2.clone())
    });

    // Verify both payouts completed
    let claim1_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim1))
    });
    let claim2_after = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<_, Claim>(&DataKey::Claim(claim2))
    });

    assert!(claim1_after.is_none());
    assert!(claim2_after.is_none());

    // Verify policies remain active
    let policy1_after = env.as_contract(&contract_id, || get_policy(env.clone(), policy1));
    let policy2_after = env.as_contract(&contract_id, || get_policy(env.clone(), policy2));

    assert!(policy1_after.active);
    assert!(policy2_after.active);
    assert_eq!(policy1_after.farmer, farmer1);
    assert_eq!(policy2_after.farmer, farmer2);
}

#[test]
fn test_end_to_end_insurance_ecosystem() {
    let env = Env::default();
    let (farmer, admin) = create_test_accounts(&env);

    env.mock_all_auths();

    let contract_id = create_test_contract(&env);

    // Test complete ecosystem flow with multiple cycles
    for cycle in 1..=3 {
        let policy_id = env.as_contract(&contract_id, || {
            insurance::create_pol(
                env.clone(),
                farmer.clone(),
                symbol_short!("ecosys"),
                cycle * 100,
            )
            .unwrap()
        });

        // Verify policy created
        let policy = env.as_contract(&contract_id, || get_policy(env.clone(), policy_id.clone()));
        assert!(!policy.active);
        assert_eq!(policy.premium, cycle * 100);

        // Pay premium
        env.as_contract(&contract_id, || {
            insurance::pay_prem(env.clone(), policy_id.clone())
        });

        // Verify policy activated
        let active_policy =
            env.as_contract(&contract_id, || get_policy(env.clone(), policy_id.clone()));
        assert!(active_policy.active);

        // Submit multiple claims per cycle
        for claim_num in 1..=cycle {
            let event_hash = BytesN::random(&env);
            let payout_amount = claim_num * 250;

            let claim_id = env.as_contract(&contract_id, || {
                claims::sub_claim(env.clone(), policy_id.clone(), event_hash, payout_amount)
                    .unwrap()
            });

            // Process payout immediately
            env.as_contract(&contract_id, || {
                payouts::pay_out(env.clone(), claim_id.clone(), admin.clone())
            });

            // Verify payout completed
            let claim_after = env.as_contract(&contract_id, || {
                env.storage()
                    .instance()
                    .get::<_, Claim>(&DataKey::Claim(claim_id))
            });
            assert!(claim_after.is_none());
        }

        // Verify policy still active after all payouts
        let final_policy = env.as_contract(&contract_id, || get_policy(env.clone(), policy_id));
        assert!(final_policy.active);
    }
}
