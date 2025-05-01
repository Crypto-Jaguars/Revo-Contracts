#[cfg(test)]
mod test {
    #![allow(deprecated)]
    use crate::{CSAMembershipContract, CSAMembershipContractClient, ShareSize};
    use soroban_sdk::{
        testutils::{Address as TestAddress, Events, MockAuth, MockAuthInvoke, Ledger},
        Env, BytesN, String, IntoVal, Vec, Val, TryFromVal, Address,
    };

    fn setup_env() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|ledger| ledger.timestamp = 86400);
        let contract_id = Address::generate(&env);
        env.register_contract(&contract_id, CSAMembershipContract);
        (env, contract_id)
    }

    #[test]
    fn test_enroll_membership() {
        let (env, contract_id) = setup_env();
        let client = CSAMembershipContractClient::new(&env, &contract_id);

        let farm_id = BytesN::from_array(&env, &[1; 32]);
        let season = String::from_str(&env, "Summer 2023");
        let share_size = ShareSize::Medium;
        let pickup_location = String::from_str(&env, "Farm Market");
        let start_date = env.ledger().timestamp() + 86400;
        let end_date = start_date + 86400 * 90;
        let member = Address::generate(&env);

        env.logs().add("test_enroll_membership", &[start_date.into_val(&env), env.ledger().timestamp().into_val(&env)]);

        env.mock_auths(&[MockAuth {
            address: &member,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "enroll_membership",
                args: (farm_id.clone(), season.clone(), share_size, pickup_location.clone(), start_date, end_date, member.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let token_id = client.enroll_membership(&farm_id, &season, &share_size, &pickup_location, &start_date, &end_date, &member);

        let enroll_events = env.events().all();
        env.logs().add("events_after_enroll", &[enroll_events.len().into_val(&env)]);
        assert_eq!(enroll_events.len(), 1, "Expected 1 event, got {}", enroll_events.len());

        let event = enroll_events.get_unchecked(0);
        let expected_topics: Vec<Val> = soroban_sdk::vec![
            &env,
            String::from_str(&env, "enroll_membership").into_val(&env),
            String::from_str(&env, "success").into_val(&env),
        ];
        assert_eq!(event.1, expected_topics, "Event topics mismatch");

        let expected_data: Vec<Val> = soroban_sdk::vec![
            &env,
            member.clone().into_val(&env),
            token_id.clone().into_val(&env),
        ];
        let event_data: Vec<Val> = Vec::try_from_val(&env, &event.2).expect("Failed to convert event data to Vec");
        assert_eq!(event_data, expected_data, "Event data mismatch");
    }

    #[test]
    fn test_update_pickup_location() {
        let (env, contract_id) = setup_env();
        let client = CSAMembershipContractClient::new(&env, &contract_id);

        let farm_id = BytesN::from_array(&env, &[1; 32]);
        let season = String::from_str(&env, "Summer 2023");
        let share_size = ShareSize::Medium;
        let pickup_location = String::from_str(&env, "Farm Market");
        let start_date = env.ledger().timestamp() + 86400;
        let end_date = start_date + 86400 * 90;
        let member = Address::generate(&env);

        env.logs().add("test_update_pickup_location", &[start_date.into_val(&env), env.ledger().timestamp().into_val(&env)]);

        env.mock_auths(&[MockAuth {
            address: &member,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "enroll_membership",
                args: (farm_id.clone(), season.clone(), share_size, pickup_location.clone(), start_date, end_date, member.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let token_id = client.enroll_membership(&farm_id, &season, &share_size, &pickup_location, &start_date, &end_date, &member);

        let enroll_events = env.events().all();
        env.logs().add("events_after_enroll", &[enroll_events.len().into_val(&env)]);
        assert_eq!(enroll_events.len(), 1, "Expected 1 event from enroll_membership, got {}", enroll_events.len());

        let new_location = String::from_str(&env, "City Market");

        env.mock_auths(&[MockAuth {
            address: &member,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "update_pickup_location",
                args: (token_id.clone(), new_location.clone(), member.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.try_update_pickup_location(&token_id, &new_location, &member);
        let _ = result.expect("Failed to update pickup location");

        let update_events = env.events().all();
        env.logs().add("events_after_update", &[update_events.len().into_val(&env)]);
        assert_eq!(update_events.len(), 1, "Expected 1 event from update_pickup_location, got {}", update_events.len());

        let updated_membership = client.get_membership_metadata(&token_id).expect("Membership not found");
        assert_eq!(updated_membership.pickup_location, new_location, "Pickup location not updated");
    }

    #[test]
    fn test_cancel_membership() {
        let (env, contract_id) = setup_env();
        let client = CSAMembershipContractClient::new(&env, &contract_id);

        let farm_id = BytesN::from_array(&env, &[1; 32]);
        let season = String::from_str(&env, "Summer 2023");
        let share_size = ShareSize::Medium;
        let pickup_location = String::from_str(&env, "Farm Market");
        let start_date = env.ledger().timestamp() + 86400;
        let end_date = start_date + 86400 * 90;
        let member = Address::generate(&env);

        env.logs().add("test_cancel_membership", &[start_date.into_val(&env), env.ledger().timestamp().into_val(&env)]);

        env.mock_auths(&[MockAuth {
            address: &member,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "enroll_membership",
                args: (farm_id.clone(), season.clone(), share_size, pickup_location.clone(), start_date, end_date, member.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let token_id = client.enroll_membership(&farm_id, &season, &share_size, &pickup_location, &start_date, &end_date, &member);

        let enroll_events = env.events().all();
        env.logs().add("events_after_enroll", &[enroll_events.len().into_val(&env)]);
        assert_eq!(enroll_events.len(), 1, "Expected 1 event from enroll_membership, got {}", enroll_events.len());

        env.mock_auths(&[MockAuth {
            address: &member,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "cancel_membership",
                args: (token_id.clone(), member.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.try_cancel_membership(&token_id, &member);
        let _ = result.expect("Failed to cancel membership");

        let cancel_events = env.events().all();
        env.logs().add("events_after_cancel", &[cancel_events.len().into_val(&env)]);
        assert_eq!(cancel_events.len(), 1, "Expected 1 event from cancel_membership, got {}", cancel_events.len());

        let membership_after_cancel = client.get_membership_metadata(&token_id);
        assert!(membership_after_cancel.is_none(), "Membership should be removed after cancel");
    }

    #[test]
    fn test_enroll_membership_invalid_dates() {
        let (env, contract_id) = setup_env();
        let client = CSAMembershipContractClient::new(&env, &contract_id);

        let farm_id = BytesN::from_array(&env, &[1; 32]);
        let season = String::from_str(&env, "Summer 2023");
        let share_size = ShareSize::Medium;
        let pickup_location = String::from_str(&env, "Farm Market");
        let start_date = env.ledger().timestamp() - 3600;
        let end_date = start_date + 86400 * 90;
        let member = Address::generate(&env);

        env.mock_auths(&[MockAuth {
            address: &member,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "enroll_membership",
                args: (farm_id.clone(), season.clone(), share_size, pickup_location.clone(), start_date, end_date, member.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }]);

        let result = client.try_enroll_membership(&farm_id, &season, &share_size, &pickup_location, &start_date, &end_date, &member);
        assert!(result.is_err(), "Expected an error due to invalid dates");
    }
}