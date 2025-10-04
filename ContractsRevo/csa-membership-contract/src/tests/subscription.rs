use crate::{tests::utils::*, Error, ShareSize};
use soroban_sdk::String;

#[test]
fn test_subscription_enrollment_tracking() {
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
    assert_eq!(membership.member, test_env.member1);
    assert_eq!(membership.start_date, FUTURE_START_DATE);
    assert_eq!(membership.end_date, FUTURE_END_DATE);
}

#[test]
fn test_subscription_status_active() {
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

    // Verify subscription exists (active)
    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_some());
}

#[test]
fn test_subscription_status_cancelled() {
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

    client.cancel_membership(&token_id, &test_env.member1);

    // Verify subscription is cancelled (no longer exists)
    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_none());
}

#[test]
fn test_subscription_period_validation() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let start_date = FUTURE_START_DATE;
    let end_date = FUTURE_END_DATE;

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &start_date,
        &end_date,
        &test_env.member1,
    );

    let membership = client.get_membership_metadata(&token_id).unwrap();
    assert!(membership.end_date > membership.start_date);
}

#[test]
fn test_subscription_with_short_period() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let current_time = test_env.env.ledger().timestamp();
    let start_date = current_time + 100;
    let end_date = start_date + 86400; // 1 day subscription

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

#[test]
fn test_subscription_with_long_period() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let current_time = test_env.env.ledger().timestamp();
    let start_date = current_time + 100;
    let end_date = start_date + (365 * 86400); // 1 year subscription

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

#[test]
fn test_multiple_subscriptions_overwrites() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // First subscription
    let token_id1 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Spring 2025"),
        &ShareSize::Small,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Second subscription for different season (will overwrite)
    let token_id2 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Fall 2025"),
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Token IDs are the same
    assert_eq!(token_id1, token_id2);

    // Only latest subscription data exists
    let membership = client.get_membership_metadata(&token_id2).unwrap();
    assert_eq!(
        membership.season,
        String::from_str(&test_env.env, "Fall 2025")
    );
    assert_eq!(membership.share_size, ShareSize::Large);
}

#[test]
fn test_subscription_cancellation_by_owner() {
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

    let result = client.try_cancel_membership(&token_id, &test_env.member1);
    assert!(result.is_ok());

    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_none());
}

#[test]
fn test_subscription_cancellation_unauthorized() {
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

    // Try to cancel by different member
    let result = client.try_cancel_membership(&token_id, &test_env.member2);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));

    // Original membership should still exist
    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_some());
}

#[test]
fn test_subscription_renewal_workflow() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Initial subscription
    let token_id1 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Summer 2025"),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    assert!(client.get_membership_metadata(&token_id1).is_some());

    // Cancel first subscription
    client.cancel_membership(&token_id1, &test_env.member1);
    assert!(client.get_membership_metadata(&token_id1).is_none());

    // Renew for next season
    let token_id2 = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Fall 2025"),
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Token IDs are the same
    assert_eq!(token_id1, token_id2);
    // New subscription should exist with updated data
    let membership = client.get_membership_metadata(&token_id2).unwrap();
    assert_eq!(
        membership.season,
        String::from_str(&test_env.env, "Fall 2025")
    );
    assert_eq!(membership.share_size, ShareSize::Large);
}

#[test]
fn test_subscription_update_during_period() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    let token_id = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &standard_season(&test_env.env),
        &ShareSize::Medium,
        &String::from_str(&test_env.env, "Downtown Market"),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Update pickup location during subscription period
    let new_location = String::from_str(&test_env.env, "Uptown Market");
    let result = client.try_update_pickup_location(&token_id, &new_location, &test_env.member1);
    assert!(result.is_ok());

    let membership = client.get_membership_metadata(&token_id).unwrap();
    assert_eq!(membership.pickup_location, new_location);
}

#[test]
fn test_subscription_multiple_share_sizes() {
    let test_env = setup_test();
    let client = create_client(&test_env);

    test_env.env.mock_all_auths();

    // Small share subscription
    let token_small = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Spring Small"),
        &ShareSize::Small,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member1,
    );

    // Medium share subscription
    let token_medium = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Spring Medium"),
        &ShareSize::Medium,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member2,
    );

    // Large share subscription
    let token_large = client.enroll_membership(
        &standard_farm_id(&test_env.env),
        &String::from_str(&test_env.env, "Spring Large"),
        &ShareSize::Large,
        &standard_pickup_location(&test_env.env),
        &FUTURE_START_DATE,
        &FUTURE_END_DATE,
        &test_env.member3,
    );

    // All token IDs are the same, last enrollment wins
    assert_eq!(token_small, token_medium);
    assert_eq!(token_medium, token_large);

    // Only the last membership exists
    let membership_large = client.get_membership_metadata(&token_large).unwrap();
    assert_eq!(membership_large.share_size, ShareSize::Large);
    assert_eq!(membership_large.member, test_env.member3);
}
