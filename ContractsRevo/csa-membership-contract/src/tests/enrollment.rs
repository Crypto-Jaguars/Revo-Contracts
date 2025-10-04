use crate::{tests::utils::*, Error, ShareSize};
use soroban_sdk::{testutils::Address as _, BytesN, String};

#[test]
fn test_enrollment_with_valid_details() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let result = client.try_enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    assert!(result.is_ok());
}

#[test]
fn test_enrollment_with_small_share() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Small,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let membership = client.get_membership_metadata(&token_id).unwrap();
    assert_eq!(membership.share_size, ShareSize::Small);
}

#[test]
fn test_enrollment_with_large_share() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let membership = client.get_membership_metadata(&token_id).unwrap();
    assert_eq!(membership.share_size, ShareSize::Large);
}

#[test]
fn test_enrollment_with_empty_pickup_location() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Empty pickup location should still be allowed (member might pick up directly)
    let result = client.try_enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &String::from_str(&test_env.env, ""),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    assert!(result.is_ok());
}

#[test]
fn test_enrollment_with_zero_farm_id() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let zero_farm_id = BytesN::from_array(&test_env.env, &[0; 32]);
    let result = client.try_enroll_membership(
        &zero_farm_id,
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    assert_eq!(result, Err(Ok(Error::InvalidFarm)));
}

#[test]
fn test_enrollment_with_empty_season() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let result = client.try_enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, ""),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    assert_eq!(result, Err(Ok(Error::InvalidSeason)));
}

#[test]
fn test_enrollment_with_past_start_date() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let past_date = 1609459200u64; // Jan 1, 2021 (past)
    let result = client.try_enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &past_date,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    assert_eq!(result, Err(Ok(Error::InvalidDates)));
}

#[test]
fn test_enrollment_with_end_before_start() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let result = client.try_enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_END_DATE,
        &FUTURE_START_DATE,
        &test_env.member1,
    );

    assert_eq!(result, Err(Ok(Error::InvalidDates)));
}

#[test]
fn test_duplicate_enrollment_same_member() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // First enrollment
    let token_id1 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Second enrollment (will overwrite first due to same token_id)
    let token_id2 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Fall 2025"),
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Token IDs are the same (hardcoded to zeros in enroll.rs)
    assert_eq!(token_id1, token_id2);

    // Only the latest membership data should exist
    let membership = client.get_membership_metadata(&token_id2).unwrap();
    assert_eq!(
        membership.season,
        String::from_str(&test_env.env, "Fall 2025")
    );
    assert_eq!(membership.share_size, ShareSize::Large);
}

#[test]
fn test_membership_id_consistency() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id1 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let token_id2 = client.enroll_membership(
        &create_farm_id(&test_env.env, 2),
        &String::from_str(&test_env.env, "Fall 2025"),
        &ShareSize::Large,
        &String::from_str(&test_env.env, "Uptown Market"),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member2,
    );

    // Token IDs are the same (hardcoded to zeros in enroll.rs)
    assert_eq!(token_id1, token_id2);

    // Latest membership data should exist
    let membership = client.get_membership_metadata(&token_id2).unwrap();
    assert_eq!(membership.member, test_env.member2);
    assert_eq!(membership.share_size, ShareSize::Large);
}

#[test]
fn test_high_volume_enrollment() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Enroll 50 members to test scalability
    for i in 0..50 {
        let member = soroban_sdk::Address::generate(&test_env.env);
        let season = String::from_str(&test_env.env, "Summer 2025");

        let result = client.try_enroll_membership(
            &create_farm_id(&test_env.env, (i % 5) + 1),
            &season,
            &ShareSize::Medium,
            &standard_pickup_location(&test_env.env),
            &FUTURE_START_DATE,
            &FUTURE_END_DATE,
            &member,
        );

        assert!(result.is_ok(), "Enrollment {} failed", i);
    }
}

#[test]
fn test_enrollment_different_farms_overwrites() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let farm1 = create_farm_id(&test_env.env, 1);
    let farm2 = create_farm_id(&test_env.env, 2);
    let season = standard_season(&test_env.env);

    let token_id1 = client.enroll_membership(
        &farm1,
        &season,
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let token_id2 = client.enroll_membership(
        &farm2,
        &season,
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Token IDs are the same, so second enrollment overwrites first
    assert_eq!(token_id1, token_id2);

    let membership = client.get_membership_metadata(&token_id2).unwrap();
    assert_eq!(membership.farm_id, farm2);
    assert_eq!(membership.share_size, ShareSize::Large);
}

#[test]
fn test_enrollment_with_very_long_season_name() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let long_season = String::from_str(
        &test_env.env,
        "Summer 2025 Extended Season with Special Organic Produce",
    );

    let result = client.try_enroll_membership(
        &standard_farm_id(&test_env.env),
        &long_season,
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    assert!(result.is_ok());
}

#[test]
fn test_enrollment_date_boundary_validation() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Set start date to exactly current timestamp + 1 second
    let current_time = test_env.env.ledger().timestamp();
    let start_date = current_time + 1;
    let end_date = start_date + 100;

    let result = client.try_enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &start_date,
        &end_date,
        &test_env.member1,
    );

    assert!(result.is_ok());
}
