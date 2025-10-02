use crate::datatype::{CooperativeError, DataKey, Member};
use crate::interface::Membership;
use crate::tests::utils::*;
use crate::CooperativeManagementContract;
use soroban_sdk::{testutils::Address as _, Address, String};

#[test]
fn test_register_member_success() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    assert!(result.is_ok());

    // Verify member was registered
    let stored_member = test_env.env.as_contract(&test_env.contract_id, || {
        let member_key = DataKey::Member(test_env.member1.clone());
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&member_key)
            .unwrap()
    });

    assert_eq!(stored_member.name, standard_member_name(&test_env.env));
    assert_eq!(stored_member.role, standard_farmer_role(&test_env.env));
    assert_eq!(stored_member.verified, false);
    assert_eq!(stored_member.reputation, 0);
    assert_eq!(stored_member.contributions, 0);
}

#[test]
fn test_register_member_duplicate() {
    let test_env = setup_test();

    // First registration
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Try to register the same member again
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Different Name"),
            standard_farmer_role(&test_env.env),
        )
    });

    assert_eq!(result, Err(CooperativeError::MemberAlreadyExists));
}

#[test]
fn test_register_member_with_different_roles() {
    let test_env = setup_test();

    // Register farmer
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Alice"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register manager
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Bob"),
            standard_manager_role(&test_env.env),
        )
    });

    // Verify both members
    let member1 = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });

    let member2 = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member2.clone()))
            .unwrap()
    });

    assert_eq!(member1.role, standard_farmer_role(&test_env.env));
    assert_eq!(member2.role, standard_manager_role(&test_env.env));
}

#[test]
fn test_verify_member_success() {
    let test_env = setup_test();

    // Register member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Verify member
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::verify_member(
            test_env.env.clone(),
            test_env.admin.clone(),
            test_env.member1.clone(),
        )
    });

    assert!(result.is_ok());

    // Check verification status
    let member = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(member.verified, true);
}

#[test]
fn test_verify_member_not_found() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::verify_member(
            test_env.env.clone(),
            test_env.admin.clone(),
            test_env.member1.clone(),
        )
    });

    assert_eq!(result, Err(CooperativeError::MemberNotFound));
}

#[test]
fn test_track_contribution_success() {
    let test_env = setup_test();

    // Register member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Track contribution
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::track_contribution(
            test_env.env.clone(),
            test_env.member1.clone(),
            100,
        )
    });

    assert!(result.is_ok());

    // Verify contribution
    let member = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(member.contributions, 100);
}

#[test]
fn test_track_contribution_multiple() {
    let test_env = setup_test();

    // Register member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Track multiple contributions
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::track_contribution(
            test_env.env.clone(),
            test_env.member1.clone(),
            100,
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::track_contribution(
            test_env.env.clone(),
            test_env.member1.clone(),
            50,
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::track_contribution(
            test_env.env.clone(),
            test_env.member1.clone(),
            75,
        )
    });

    // Verify total contributions
    let member = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(member.contributions, 225);
}

#[test]
fn test_track_contribution_not_found() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::track_contribution(
            test_env.env.clone(),
            test_env.member1.clone(),
            100,
        )
    });

    assert_eq!(result, Err(CooperativeError::MemberNotFound));
}

#[test]
fn test_update_reputation_success() {
    let test_env = setup_test();

    // Register member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Update reputation
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::update_reputation(
            test_env.env.clone(),
            test_env.admin.clone(),
            test_env.member1.clone(),
            50,
        )
    });

    assert!(result.is_ok());

    // Verify reputation
    let member = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(member.reputation, 50);
}

#[test]
fn test_update_reputation_multiple() {
    let test_env = setup_test();

    // Register member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Update reputation multiple times
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::update_reputation(
            test_env.env.clone(),
            test_env.admin.clone(),
            test_env.member1.clone(),
            10,
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::update_reputation(
            test_env.env.clone(),
            test_env.admin.clone(),
            test_env.member1.clone(),
            20,
        )
    });

    // Verify total reputation
    let member = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(member.reputation, 30);
}

#[test]
fn test_update_reputation_not_found() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::update_reputation(
            test_env.env.clone(),
            test_env.admin.clone(),
            test_env.member1.clone(),
            50,
        )
    });

    assert_eq!(result, Err(CooperativeError::MemberNotFound));
}

#[test]
fn test_high_volume_member_registration() {
    let test_env = setup_test();

    // Register 30 members to test scalability
    for i in 0..30 {
        let member = Address::generate(&test_env.env);
        let name = String::from_str(&test_env.env, "Member");
        let role = if i % 2 == 0 {
            standard_farmer_role(&test_env.env)
        } else {
            standard_manager_role(&test_env.env)
        };

        let result = test_env.env.as_contract(&test_env.contract_id, || {
            <CooperativeManagementContract as Membership>::register_member(
                test_env.env.clone(),
                member.clone(),
                name.clone(),
                role.clone(),
            )
        });

        assert!(result.is_ok(), "Registration {} failed", i);
    }
}

#[test]
fn test_member_identity_uniqueness() {
    let test_env = setup_test();

    // Register three different members
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Alice"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Bob"),
            standard_manager_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member3.clone(),
            String::from_str(&test_env.env, "Charlie"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Verify each member has unique identity
    let member1 = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });

    let member2 = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member2.clone()))
            .unwrap()
    });

    let member3 = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member3.clone()))
            .unwrap()
    });

    assert_eq!(member1.address, test_env.member1);
    assert_eq!(member2.address, test_env.member2);
    assert_eq!(member3.address, test_env.member3);
    assert_eq!(member1.name, String::from_str(&test_env.env, "Alice"));
    assert_eq!(member2.name, String::from_str(&test_env.env, "Bob"));
    assert_eq!(member3.name, String::from_str(&test_env.env, "Charlie"));
}

#[test]
fn test_member_verification_workflow() {
    let test_env = setup_test();

    // Register member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Check initial state
    let member = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });
    assert_eq!(member.verified, false);

    // Verify member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::verify_member(
            test_env.env.clone(),
            test_env.admin.clone(),
            test_env.member1.clone(),
        )
    });

    // Check verified state
    let member = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&DataKey::Member(test_env.member1.clone()))
            .unwrap()
    });
    assert_eq!(member.verified, true);
}
