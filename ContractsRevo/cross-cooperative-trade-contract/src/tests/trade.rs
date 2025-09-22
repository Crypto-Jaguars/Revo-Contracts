#![cfg(test)]

use super::*;
use crate::tests::utils::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

mod trade_offer_creation {
    use super::*;

    #[test]
    fn test_create_trade_offer_success() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        let offer_id = client
            .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // Verify trade offer was created correctly
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");

        assert_trade_offer_matches(
            &trade_offer,
            &offer_id,
            &cooperative,
            &offered_product,
            &requested_product,
            "Pending",
            &env,
        );
    }

    #[test]
    fn test_create_trade_offer_invalid_same_products() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative = Address::generate(&env);
        let same_product = create_test_product(&env, "wheat");

        let result = client.try_create_trade_offer(&cooperative, &same_product, &same_product);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected validation error for same products"
                );
            }
            Err(_) => {
                // This is also acceptable - the validation error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_create_trade_offer_unauthorized_cooperative() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let unauthorized_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Don't mock auth for this cooperative
        env.mock_auths(&[]);

        let result = client.try_create_trade_offer(
            &unauthorized_cooperative,
            &offered_product,
            &requested_product,
        );

        assert_is_error(result);
    }

    #[test]
    fn test_create_multiple_trade_offers() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperatives = create_test_cooperatives(&env, 3);

        let mut offer_ids = Vec::new(&env);

        for (i, cooperative) in cooperatives.iter().enumerate() {
            let offered_product =
                create_test_product(&env, &["product_a", "product_b", "product_c"][i % 3]);
            let requested_product =
                create_test_product(&env, &["requested_a", "requested_b", "requested_c"][i % 3]);

            let offer_id = client
                .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
                .unwrap()
                .expect("Trade offer creation should succeed");

            offer_ids.push_back(offer_id);
        }

        // Verify all offers are in active offers list
        let active_offers = client
            .try_list_active_offers()
            .unwrap()
            .expect("Should be able to list active offers");

        assert_eq!(active_offers.len(), 3);
        for i in 0..offer_ids.len() {
            assert!(active_offers.contains(&offer_ids.get(i).unwrap()));
        }
    }

    #[test]
    fn test_create_trade_offer_duplicate_products_different_cooperatives() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative1 = Address::generate(&env);
        let cooperative2 = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // First cooperative creates offer
        let offer_id1 = client
            .try_create_trade_offer(&cooperative1, &offered_product, &requested_product)
            .unwrap()
            .expect("First trade offer creation should succeed");

        // Second cooperative creates similar offer (should be allowed)
        let offer_id2 = client
            .try_create_trade_offer(&cooperative2, &offered_product, &requested_product)
            .unwrap()
            .expect("Second trade offer creation should succeed");

        assert_ne!(offer_id1, offer_id2, "Offer IDs should be different");

        // Verify both offers exist
        let active_offers = client
            .try_list_active_offers()
            .unwrap()
            .expect("Should be able to list active offers");
        assert_eq!(active_offers.len(), 2);
    }
}

mod trade_offer_acceptance {
    use super::*;

    #[test]
    fn test_accept_trade_success() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // Accept trade offer
        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        // Verify trade offer status was updated
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");
        assert_eq!(trade_offer.status, String::from_str(&env, "Accepted"));

        // Verify barter agreement was created
        let barter_agreement = client
            .try_get_barter_agreement(&agreement_id)
            .unwrap()
            .expect("Barter agreement should exist");

        assert_barter_agreement_matches(
            &barter_agreement,
            &agreement_id,
            &offer_id,
            &offering_cooperative,
            &accepting_cooperative,
            "Active",
            &env,
        );
    }

    #[test]
    fn test_accept_trade_own_offer() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create trade offer
        let offer_id = client
            .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // Try to accept own offer (should fail)
        let result = client.try_accept_trade(&offer_id, &cooperative);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error when accepting own offer"
                );
            }
            Err(_) => {
                // This is also acceptable - the validation error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_accept_trade_nonexistent_offer() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let accepting_cooperative = Address::generate(&env);
        let nonexistent_offer_id = create_test_product(&env, "nonexistent");

        let result = client.try_accept_trade(&nonexistent_offer_id, &accepting_cooperative);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error for non-existent offer"
                );
            }
            Err(_) => {
                // This is also acceptable - the error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_accept_trade_already_accepted() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative1 = Address::generate(&env);
        let accepting_cooperative2 = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // First acceptance should succeed
        let _agreement_id1 = client
            .try_accept_trade(&offer_id, &accepting_cooperative1)
            .unwrap()
            .expect("First accept trade should succeed");

        // Second acceptance should fail
        let result = client.try_accept_trade(&offer_id, &accepting_cooperative2);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error when accepting already accepted offer"
                );
            }
            Err(_) => {
                // This is also acceptable - the validation error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_accept_trade_unauthorized_cooperative() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let unauthorized_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // Don't mock auth for unauthorized cooperative
        env.mock_auths(&[]);

        let result = client.try_accept_trade(&offer_id, &unauthorized_cooperative);

        assert_is_error(result);
    }
}

mod trade_completion {
    use super::*;

    #[test]
    fn test_complete_trade_success() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create and accept trade offer
        let (offer_id, _agreement_id) = create_complete_trade_flow(
            &env,
            &client,
            &offering_cooperative,
            &accepting_cooperative,
            &offered_product,
            &requested_product,
        );

        // Verify trade offer status was updated to Completed
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");
        assert_eq!(trade_offer.status, String::from_str(&env, "Completed"));
    }

    #[test]
    fn test_complete_trade_before_acceptance() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create trade offer but don't accept it
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // Try to complete trade before acceptance (should fail)
        let result = client.try_complete_trade(&offer_id, &offering_cooperative);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error when completing non-accepted trade"
                );
            }
            Err(_) => {
                // This is also acceptable - the validation error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_complete_trade_unauthorized_caller() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let unauthorized_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create and accept trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        let _agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        // Try to complete trade with unauthorized caller (should fail)
        let result = client.try_complete_trade(&offer_id, &unauthorized_cooperative);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error when unauthorized user tries to complete trade"
                );
            }
            Err(_) => {
                // This is also acceptable - the authorization error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_complete_trade_nonexistent_offer() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let nonexistent_offer_id = create_test_product(&env, "nonexistent");

        let result = client.try_complete_trade(&nonexistent_offer_id, &offering_cooperative);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error for non-existent trade offer"
                );
            }
            Err(_) => {
                // This is also acceptable - the error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_complete_trade_already_completed() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create, accept, and complete trade offer
        let (offer_id, _agreement_id) = create_complete_trade_flow(
            &env,
            &client,
            &offering_cooperative,
            &accepting_cooperative,
            &offered_product,
            &requested_product,
        );

        // Try to complete trade again (should fail)
        let result = client.try_complete_trade(&offer_id, &offering_cooperative);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error when completing already completed trade"
                );
            }
            Err(_) => {
                // This is also acceptable - the validation error could cause the call to fail
            }
        }
    }
}

mod trade_queries {
    use super::*;

    #[test]
    fn test_get_trade_details_success() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        let offer_id = client
            .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");

        assert_trade_offer_matches(
            &trade_offer,
            &offer_id,
            &cooperative,
            &offered_product,
            &requested_product,
            "Pending",
            &env,
        );
    }

    #[test]
    fn test_get_trade_details_not_found() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let nonexistent_offer_id = create_test_product(&env, "nonexistent");

        let result = client.try_get_trade_details(&nonexistent_offer_id);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error for non-existent trade offer"
                );
            }
            Err(_) => {
                // This is also acceptable - the error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_list_active_offers_empty() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);

        let active_offers = client
            .try_list_active_offers()
            .unwrap()
            .expect("Should be able to list active offers");

        assert_eq!(
            active_offers.len(),
            0,
            "Should have no active offers initially"
        );
    }

    #[test]
    fn test_list_active_offers_with_offers() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);

        // Create multiple trade offers
        let offers = setup_multiple_trade_offers(&env, &client, 3);

        let active_offers = client
            .try_list_active_offers()
            .unwrap()
            .expect("Should be able to list active offers");

        assert_eq!(active_offers.len(), 3, "Should have 3 active offers");
        for i in 0..offers.len() {
            let (offer_id, _, _, _) = offers.get(i).unwrap();
            assert!(active_offers.contains(offer_id), "Should contain offer");
        }
    }

    #[test]
    fn test_list_active_offers_after_acceptance() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let accepting_cooperative = Address::generate(&env);

        // Create multiple trade offers
        let offers = setup_multiple_trade_offers(&env, &client, 3);

        // Accept one of the offers
        let (first_offer_id, _, _, _) = offers.get(0).unwrap();
        let _agreement_id = client
            .try_accept_trade(&first_offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        let active_offers = client
            .try_list_active_offers()
            .unwrap()
            .expect("Should be able to list active offers");

        assert_eq!(
            active_offers.len(),
            2,
            "Should have 2 active offers after acceptance"
        );
        assert!(
            !active_offers.contains(first_offer_id),
            "Should not contain accepted offer"
        );

        let (second_offer_id, _, _, _) = offers.get(1).unwrap();
        let (third_offer_id, _, _, _) = offers.get(2).unwrap();
        assert!(
            active_offers.contains(second_offer_id),
            "Should contain second offer"
        );
        assert!(
            active_offers.contains(third_offer_id),
            "Should contain third offer"
        );
    }
}
