use crate::{tests::utils::*, Error, ShareSize};
use soroban_sdk::String;

#[test]
fn test_benefit_eligibility_active_member() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Verify member is enrolled and eligible for benefits
    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_some());

    let membership = membership.unwrap();
    assert_eq!(membership.member, test_env.member1);
    assert_eq!(membership.share_size, ShareSize::Medium);
}

#[test]
fn test_benefit_eligibility_cancelled_member() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Cancel membership
    client.cancel_membership(&token_id, &test_env.member1);

    // Member should no longer be eligible (membership doesn't exist)
    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_none());
}

#[test]
fn test_benefit_eligibility_non_subscribed_member() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Try to check benefits for a member who never enrolled
    let fake_token_id = create_farm_id(&test_env.env, 99);
    let membership = client.get_membership_metadata(&fake_token_id);
    assert!(membership.is_none());
}

#[test]
fn test_benefit_distribution_small_share() {
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
fn test_benefit_distribution_medium_share() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let membership = client.get_membership_metadata(&token_id).unwrap();
    assert_eq!(membership.share_size, ShareSize::Medium);
}

#[test]
fn test_benefit_distribution_large_share() {
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
fn test_benefit_tracking_overwrites_by_season() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let summer_season = String::from_str(&test_env.env, "Summer 2025");
    let fall_season = String::from_str(&test_env.env, "Fall 2025");

    let token_summer = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &summer_season,
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let token_fall = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &fall_season,
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member2,
    );

    // Token IDs are the same, last enrollment wins
    assert_eq!(token_summer, token_fall);

    let membership = client.get_membership_metadata(&token_fall).unwrap();
    assert_eq!(membership.season, fall_season);
    assert_eq!(membership.member, test_env.member2);
}

#[test]
fn test_benefit_tracking_overwrites_by_farm() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let farm1 = create_farm_id(&test_env.env, 1);
    let farm2 = create_farm_id(&test_env.env, 2);

    let token1 = client.enroll_membership(
        &farm1,
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let token2 = client.enroll_membership(
        &farm2,
        &standard_season(&test_env.env),
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member2,
    );

    // Token IDs are the same, last enrollment wins
    assert_eq!(token1, token2);

    let membership = client.get_membership_metadata(&token2).unwrap();
    assert_eq!(membership.farm_id, farm2);
    assert_eq!(membership.member, test_env.member2);
}

#[test]
fn test_benefit_pickup_location_tracking() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let location1 = String::from_str(&test_env.env, "North Market");
    let location2 = String::from_str(&test_env.env, "South Market");

    let token1 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Season 1"),
        &ShareSize::Medium,
        &location1,
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    let token2 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Season 2"),
        &ShareSize::Medium,
        &location2,
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member2,
    );

    // Token IDs are the same, last enrollment wins
    assert_eq!(token1, token2);

    let membership = client.get_membership_metadata(&token2).unwrap();
    assert_eq!(membership.pickup_location, location2);
    assert_eq!(membership.member, test_env.member2);
}

#[test]
fn test_benefit_dispute_unauthorized_access() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Try to update pickup location as different member (dispute scenario)
    let result = client.try_update_pickup_location(
        &token_id,
        &String::from_str(&test_env.env, "Fake Location"),
        &test_env.member2,
    );

    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_benefit_dispute_invalid_token() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Try to access benefits with invalid token
    let fake_token = create_farm_id(&test_env.env, 99);
    let result = client.try_update_pickup_location(
        &fake_token,
        &String::from_str(&test_env.env, "New Location"),
        &test_env.member1,
    );

    assert_eq!(result, Err(Ok(Error::NotFound)));
}

#[test]
fn test_multiple_members_benefit_distribution() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Enroll multiple members with different share sizes
    let members_and_shares = [
        (test_env.member1.clone(), ShareSize::Small, "Season 1"),
        (test_env.member2.clone(), ShareSize::Medium, "Season 2"),
        (test_env.member3.clone(), ShareSize::Large, "Season 3"),
    ];

    for (member, share_size, season_name) in members_and_shares.iter() {
        let season = String::from_str(&test_env.env, season_name);
        let token_id = client.enroll_membership(
            &standard_farm_id(&test_env.env),
            &season,
            share_size,
            &standard_pickup_location(&test_env.env),
            &FUTURE_START_DATE,
            &FUTURE_END_DATE,
            member,
        );

        let membership = client.get_membership_metadata(&token_id).unwrap();
        assert_eq!(membership.member, *member);
        assert_eq!(membership.share_size, *share_size);
    }
}
