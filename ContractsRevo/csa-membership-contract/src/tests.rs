use soroban_sdk::{testutils::{Address as _, Ledger}, Address, BytesN, Env, String};
use crate::{CSAMembershipContract, CSAMembershipContractClient, Error, ShareSize};

fn create_test_env() -> (Env, Address) {
    let env = Env::default();
    env.ledger().with_mut(|li| {
        li.timestamp = 1700000000; // Set current time to Nov 2023
    });
    let contract_id = env.register(CSAMembershipContract, ());
    (env, contract_id)
}

#[test]
fn test_enrollment_success() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, "Summer 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64; // Jan 1, 2025
    let end_date = 1743465600u64;   // Apr 1, 2025
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let result = client.try_enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    assert!(result.is_ok(), "Enrollment failed: {:?}", result);
    let token_id = result.unwrap().unwrap();
    
    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_some());
    
    let membership = membership.unwrap();
    assert_eq!(membership.farm_id, farm_id);
    assert_eq!(membership.season, season);
    assert_eq!(membership.share_size, ShareSize::Medium);
    assert_eq!(membership.pickup_location, pickup_location);
    assert_eq!(membership.start_date, start_date);
    assert_eq!(membership.end_date, end_date);
    assert_eq!(membership.member, member);
}

#[test]
fn test_enrollment_different_share_sizes() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, "Summer 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    // Test Small share
    let token_id_small = client.enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Small,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    
    let membership_small = client.get_membership_metadata(&token_id_small).unwrap();
    assert_eq!(membership_small.share_size, ShareSize::Small);
    
    // Test Large share
    let token_id_large = client.enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Large,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    
    let membership_large = client.get_membership_metadata(&token_id_large).unwrap();
    assert_eq!(membership_large.share_size, ShareSize::Large);
}

#[test]
fn test_enrollment_invalid_dates() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, "Summer 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    // Test with past start date
    let past_start = 1609459200u64; // Jan 1, 2021 (past)
    let future_end = 1743465600u64; // Apr 1, 2025
    
    let result = client.try_enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &past_start,
        &future_end,
        &member,
    );
    assert_eq!(result, Err(Ok(Error::InvalidDates)));
    
    // Test with end date before start date
    let start_date = 1743465600u64; // Apr 1, 2025
    let end_date = 1735689600u64;   // Jan 1, 2025 (before start)
    
    let result = client.try_enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    assert_eq!(result, Err(Ok(Error::InvalidDates)));
}

#[test]
fn test_enrollment_invalid_farm() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[0; 32]); // Empty farm_id
    let season = String::from_str(&env, "Summer 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let result = client.try_enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    assert_eq!(result, Err(Ok(Error::InvalidFarm)));
}

#[test]
fn test_enrollment_invalid_season() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, ""); // Empty season
    let pickup_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let result = client.try_enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    assert_eq!(result, Err(Ok(Error::InvalidSeason)));
}

#[test]
fn test_update_pickup_location_success() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, "Summer 2025");
    let initial_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let token_id = client.enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &initial_location,
        &start_date,
        &end_date,
        &member,
    );
    
    let new_location = String::from_str(&env, "Uptown Farmers Market");
    client.update_pickup_location(&token_id, &new_location, &member);
    
    let updated_membership = client.get_membership_metadata(&token_id).unwrap();
    assert_eq!(updated_membership.pickup_location, new_location);
}

#[test]
fn test_update_pickup_location_unauthorized() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, "Summer 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    let unauthorized_member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let token_id = client.enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    
    let new_location = String::from_str(&env, "Uptown Farmers Market");
    let result = client.try_update_pickup_location(&token_id, &new_location, &unauthorized_member);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_update_pickup_location_not_found() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let token_id = BytesN::from_array(&env, &[99; 32]); // Non-existent token
    let new_location = String::from_str(&env, "Uptown Farmers Market");
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let result = client.try_update_pickup_location(&token_id, &new_location, &member);
    assert_eq!(result, Err(Ok(Error::NotFound)));
}

#[test]
fn test_cancel_membership_success() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, "Summer 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let token_id = client.enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    
    client.cancel_membership(&token_id, &member);
    
    let membership = client.get_membership_metadata(&token_id);
    assert!(membership.is_none());
}

#[test]
fn test_cancel_membership_unauthorized() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let season = String::from_str(&env, "Summer 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    let unauthorized_member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let token_id = client.enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    
    let result = client.try_cancel_membership(&token_id, &unauthorized_member);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_cancel_membership_not_found() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let token_id = BytesN::from_array(&env, &[99; 32]); // Non-existent token
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let result = client.try_cancel_membership(&token_id, &member);
    assert_eq!(result, Err(Ok(Error::NotFound)));
}

#[test]
fn test_seasonal_membership_workflow() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[1; 32]);
    let summer_season = String::from_str(&env, "Summer 2025");
    let fall_season = String::from_str(&env, "Fall 2025");
    let pickup_location = String::from_str(&env, "Downtown Market");
    let member = Address::generate(&env);
    
    // Summer season dates
    let summer_start = 1735689600u64; // Jan 1, 2025
    let summer_end = 1743465600u64;   // Apr 1, 2025
    
    // Fall season dates
    let fall_start = 1751328000u64;   // Jul 1, 2025
    let fall_end = 1759104000u64;     // Oct 1, 2025
    
    env.mock_all_auths();
    
    // Enroll in summer season
    let summer_token = client.enroll_membership(
        &farm_id,
        &summer_season,
        &ShareSize::Medium,
        &pickup_location,
        &summer_start,
        &summer_end,
        &member,
    );
    
    // Verify summer membership exists
    let summer_membership = client.get_membership_metadata(&summer_token).unwrap();
    assert_eq!(summer_membership.season, summer_season);
    assert_eq!(summer_membership.share_size, ShareSize::Medium);
    
    // Update pickup location for summer season
    let new_location = String::from_str(&env, "Uptown Farmers Market");
    client.update_pickup_location(&summer_token, &new_location, &member);
    
    let updated_summer_membership = client.get_membership_metadata(&summer_token).unwrap();
    assert_eq!(updated_summer_membership.pickup_location, new_location);
    
    // Cancel summer membership
    client.cancel_membership(&summer_token, &member);
    
    // Verify summer is cancelled
    let summer_after_cancel = client.get_membership_metadata(&summer_token);
    assert!(summer_after_cancel.is_none());
    
    // Now enroll in fall season (separate test since token IDs are not unique)
    let fall_token = client.enroll_membership(
        &farm_id,
        &fall_season,
        &ShareSize::Large,
        &pickup_location,
        &fall_start,
        &fall_end,
        &member,
    );
    
    // Verify fall membership exists
    let fall_membership = client.get_membership_metadata(&fall_token).unwrap();
    assert_eq!(fall_membership.season, fall_season);
    assert_eq!(fall_membership.share_size, ShareSize::Large);
}

#[test]
fn test_membership_metadata_accuracy() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id = BytesN::from_array(&env, &[42; 32]);
    let season = String::from_str(&env, "Winter 2025");
    let pickup_location = String::from_str(&env, "Central Plaza");
    let start_date = 1767225600u64; // Jan 1, 2026
    let end_date = 1774915200u64;   // Apr 1, 2026
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    let token_id = client.enroll_membership(
        &farm_id,
        &season,
        &ShareSize::Medium,
        &pickup_location,
        &start_date,
        &end_date,
        &member,
    );
    
    let membership = client.get_membership_metadata(&token_id).unwrap();
    
    // Verify all metadata fields are accurate
    assert_eq!(membership.farm_id, farm_id);
    assert_eq!(membership.season, season);
    assert_eq!(membership.share_size, ShareSize::Medium);
    assert_eq!(membership.pickup_location, pickup_location);
    assert_eq!(membership.start_date, start_date);
    assert_eq!(membership.end_date, end_date);
    assert_eq!(membership.member, member);
}

#[test]
fn test_multiple_farms_same_member() {
    let (env, contract_id) = create_test_env();
    let client = CSAMembershipContractClient::new(&env, &contract_id);
    
    let farm_id1 = BytesN::from_array(&env, &[1; 32]);
    let farm_id2 = BytesN::from_array(&env, &[2; 32]);
    let season = String::from_str(&env, "Spring 2025");
    let pickup_location1 = String::from_str(&env, "North Market");
    let pickup_location2 = String::from_str(&env, "South Market");
    let start_date = 1735689600u64;
    let end_date = 1743465600u64;
    let member = Address::generate(&env);
    
    env.mock_all_auths();
    
    // Test enrollment with first farm
    let token1 = client.enroll_membership(
        &farm_id1,
        &season,
        &ShareSize::Small,
        &pickup_location1,
        &start_date,
        &end_date,
        &member,
    );
    
    // Verify first farm membership
    let membership1 = client.get_membership_metadata(&token1).unwrap();
    assert_eq!(membership1.farm_id, farm_id1);
    assert_eq!(membership1.share_size, ShareSize::Small);
    assert_eq!(membership1.pickup_location, pickup_location1);
    assert_eq!(membership1.member, member);
    
    // Cancel first membership to test second farm separately
    client.cancel_membership(&token1, &member);
    
    // Test enrollment with second farm
    let token2 = client.enroll_membership(
        &farm_id2,
        &season,
        &ShareSize::Large,
        &pickup_location2,
        &start_date,
        &end_date,
        &member,
    );
    
    // Verify second farm membership
    let membership2 = client.get_membership_metadata(&token2).unwrap();
    assert_eq!(membership2.farm_id, farm_id2);
    assert_eq!(membership2.share_size, ShareSize::Large);
    assert_eq!(membership2.pickup_location, pickup_location2);
    assert_eq!(membership2.member, member);
}