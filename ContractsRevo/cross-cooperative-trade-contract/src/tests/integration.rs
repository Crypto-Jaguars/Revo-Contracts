#![cfg(test)]

use super::*;
use crate::tests::utils::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

mod scalability_tests {
    use super::*;

    #[test]
    fn test_multiple_simultaneous_trades() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let accepting_cooperative = Address::generate(&env);

        // Create multiple trade offers simultaneously
        let num_trades = 10;
        let mut offer_ids = Vec::new(&env);
        let mut cooperatives = Vec::new(&env);

        for i in 0..num_trades {
            let offering_cooperative = Address::generate(&env);
            let offered_product = create_test_product(
                &env,
                &[
                    "product_a",
                    "product_b",
                    "product_c",
                    "product_d",
                    "product_e",
                ][(i as usize) % 5],
            );
            let requested_product = create_test_product(
                &env,
                &[
                    "requested_a",
                    "requested_b",
                    "requested_c",
                    "requested_d",
                    "requested_e",
                ][(i as usize) % 5],
            );

            let offer_id = client
                .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
                .unwrap()
                .expect("Trade offer creation should succeed");

            offer_ids.push_back(offer_id);
            cooperatives.push_back(offering_cooperative);
        }

        // Accept all trade offers
        let mut agreement_ids = Vec::new(&env);
        for i in 0..offer_ids.len() {
            let offer_id = offer_ids.get(i).unwrap();
            let agreement_id = client
                .try_accept_trade(&offer_id, &accepting_cooperative)
                .unwrap()
                .expect("Accept trade should succeed");
            agreement_ids.push_back(agreement_id);
        }

        // Complete all trades
        for i in 0..offer_ids.len() {
            let offer_id = offer_ids.get(i).unwrap();
            let cooperative = cooperatives.get(i).unwrap();
            let result = client.try_complete_trade(&offer_id, &cooperative);
            assert_is_success(result);
        }

        // Verify all trades are completed
        for i in 0..offer_ids.len() {
            let offer_id = offer_ids.get(i).unwrap();
            let trade_offer = client
                .try_get_trade_details(&offer_id)
                .unwrap()
                .expect("Trade offer should exist");
            assert_eq!(trade_offer.status, String::from_str(&env, "Completed"));
        }

        // Verify all barter agreements exist
        for i in 0..agreement_ids.len() {
            let agreement_id = agreement_ids.get(i).unwrap();
            let barter_agreement = client
                .try_get_barter_agreement(&agreement_id)
                .unwrap()
                .expect("Barter agreement should exist");
            assert_eq!(barter_agreement.status, String::from_str(&env, "Active"));
        }
    }

    #[test]
    fn test_high_volume_trade_creation() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);

        // Create a large number of trade offers
        let num_offers = 50;
        let mut offer_ids = Vec::new(&env);

        for i in 0..num_offers {
            let cooperative = Address::generate(&env);
            let offered_product = create_test_product(
                &env,
                &[
                    "product_a",
                    "product_b",
                    "product_c",
                    "product_d",
                    "product_e",
                ][(i as usize) % 5],
            );
            let requested_product = create_test_product(
                &env,
                &[
                    "requested_a",
                    "requested_b",
                    "requested_c",
                    "requested_d",
                    "requested_e",
                ][(i as usize) % 5],
            );

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

        assert_eq!(active_offers.len(), num_offers as u32);

        // Verify all offers can be retrieved
        for i in 0..offer_ids.len() {
            let offer_id = offer_ids.get(i).unwrap();
            let trade_offer = client
                .try_get_trade_details(&offer_id)
                .unwrap()
                .expect("Trade offer should exist");
            assert_eq!(trade_offer.offer_id, offer_id);
        }
    }

    #[test]
    fn test_concurrent_trade_acceptance() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let accepting_cooperative = Address::generate(&env);

        // Create multiple trade offers
        let num_offers = 5;
        let mut offer_ids = Vec::new(&env);
        let mut cooperatives = Vec::new(&env);

        for i in 0..num_offers {
            let offering_cooperative = Address::generate(&env);
            let offered_product = create_test_product(
                &env,
                &[
                    "product_a",
                    "product_b",
                    "product_c",
                    "product_d",
                    "product_e",
                ][(i as usize) % 5],
            );
            let requested_product = create_test_product(
                &env,
                &[
                    "requested_a",
                    "requested_b",
                    "requested_c",
                    "requested_d",
                    "requested_e",
                ][(i as usize) % 5],
            );

            let offer_id = client
                .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
                .unwrap()
                .expect("Trade offer creation should succeed");

            offer_ids.push_back(offer_id);
            cooperatives.push_back(offering_cooperative);
        }

        // Accept all trade offers concurrently
        let mut agreement_ids = Vec::new(&env);
        for i in 0..offer_ids.len() {
            let offer_id = offer_ids.get(i).unwrap();
            let agreement_id = client
                .try_accept_trade(&offer_id, &accepting_cooperative)
                .unwrap()
                .expect("Accept trade should succeed");
            agreement_ids.push_back(agreement_id);
        }

        // Verify all offers are accepted
        for i in 0..offer_ids.len() {
            let offer_id = offer_ids.get(i).unwrap();
            let trade_offer = client
                .try_get_trade_details(&offer_id)
                .unwrap()
                .expect("Trade offer should exist");
            assert_eq!(trade_offer.status, String::from_str(&env, "Accepted"));
        }

        // Verify all barter agreements are created
        assert_eq!(agreement_ids.len(), num_offers as u32);
        for i in 0..agreement_ids.len() {
            let agreement_id = agreement_ids.get(i).unwrap();
            let barter_agreement = client
                .try_get_barter_agreement(&agreement_id)
                .unwrap()
                .expect("Barter agreement should exist");
            assert_eq!(barter_agreement.status, String::from_str(&env, "Active"));
        }
    }
}

mod edge_case_scenarios {
    use super::*;

    #[test]
    fn test_duplicate_trade_offer_creation() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create first trade offer
        let offer_id1 = client
            .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("First trade offer creation should succeed");

        // Create second trade offer with same products (should be allowed)
        let offer_id2 = client
            .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Second trade offer creation should succeed");

        // Verify both offers exist and are different
        assert_ne!(offer_id1, offer_id2);

        let trade_offer1 = client
            .try_get_trade_details(&offer_id1)
            .unwrap()
            .expect("First trade offer should exist");

        let trade_offer2 = client
            .try_get_trade_details(&offer_id2)
            .unwrap()
            .expect("Second trade offer should exist");

        assert_eq!(trade_offer1.cooperative_id, cooperative);
        assert_eq!(trade_offer2.cooperative_id, cooperative);
        assert_eq!(trade_offer1.offered_product, offered_product);
        assert_eq!(trade_offer2.offered_product, offered_product);
    }

    #[test]
    fn test_trade_execution_without_mutual_agreement() {
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

        // Try to complete trade without acceptance (should fail)
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

        // Try to complete trade with unauthorized cooperative (should fail)
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
    fn test_reputation_update_for_failed_trades() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative = Address::generate(&env);

        // Simulate failed trades by updating reputation with false
        for i in 0..5 {
            let result = client.try_update_reputation(&cooperative, &false);
            assert_is_success(result);
        }

        // Simulate successful trades
        for i in 0..3 {
            let result = client.try_update_reputation(&cooperative, &true);
            assert_is_success(result);
        }
    }

    #[test]
    fn test_invalid_product_token_in_trade_offer() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let cooperative = Address::generate(&env);

        // Test with same product as offered and requested (should fail)
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

        // Test with valid different products (should succeed)
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");
        let result =
            client.try_create_trade_offer(&cooperative, &offered_product, &requested_product);

        assert_is_success(result);
    }

    #[test]
    fn test_unauthorized_trade_offers() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let unauthorized_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Don't mock auth for unauthorized cooperative
        env.mock_auths(&[]);

        let result = client.try_create_trade_offer(
            &unauthorized_cooperative,
            &offered_product,
            &requested_product,
        );

        assert_is_error(result);
    }
}

mod complex_workflows {
    use super::*;

    #[test]
    fn test_complete_trade_workflow_with_reputation() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Step 1: Create trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // Verify offer is pending
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");
        assert_eq!(trade_offer.status, String::from_str(&env, "Pending"));

        // Step 2: Accept trade offer
        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        // Verify offer is accepted and agreement is created
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");
        assert_eq!(trade_offer.status, String::from_str(&env, "Accepted"));

        let barter_agreement = client
            .try_get_barter_agreement(&agreement_id)
            .unwrap()
            .expect("Barter agreement should exist");
        assert_eq!(barter_agreement.status, String::from_str(&env, "Active"));

        // Step 3: Complete trade
        let result = client.try_complete_trade(&offer_id, &offering_cooperative);
        assert_is_success(result);

        // Verify offer is completed
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");
        assert_eq!(trade_offer.status, String::from_str(&env, "Completed"));

        // Step 4: Update reputation
        let result = client.try_update_reputation(&offering_cooperative, &true);
        assert_is_success(result);
    }

    #[test]
    fn test_multiple_cooperatives_trading_network() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);

        // Create a network of cooperatives trading with each other
        let cooperatives = create_test_cooperatives(&env, 5);
        let mut all_offer_ids = Vec::new(&env);
        let mut all_agreement_ids = Vec::new(&env);

        // Create trade offers between different cooperatives
        for i in 0..cooperatives.len() - 1 {
            let offering_coop = cooperatives.get(i).unwrap();
            let accepting_coop = cooperatives.get(i + 1).unwrap();
            let offered_product = create_test_product(
                &env,
                &[
                    "product_a",
                    "product_b",
                    "product_c",
                    "product_d",
                    "product_e",
                ][(i as usize) % 5],
            );
            let requested_product = create_test_product(
                &env,
                &[
                    "requested_a",
                    "requested_b",
                    "requested_c",
                    "requested_d",
                    "requested_e",
                ][(i as usize) % 5],
            );

            // Create offer
            let offer_id = client
                .try_create_trade_offer(&offering_coop, &offered_product, &requested_product)
                .unwrap()
                .expect("Trade offer creation should succeed");

            // Accept offer
            let agreement_id = client
                .try_accept_trade(&offer_id, &accepting_coop)
                .unwrap()
                .expect("Accept trade should succeed");

            // Complete trade
            let result = client.try_complete_trade(&offer_id, &offering_coop);
            assert_is_success(result);

            all_offer_ids.push_back(offer_id);
            all_agreement_ids.push_back(agreement_id);
        }

        // Verify all trades are completed
        for i in 0..all_offer_ids.len() {
            let offer_id = all_offer_ids.get(i).unwrap();
            let trade_offer = client
                .try_get_trade_details(&offer_id)
                .unwrap()
                .expect("Trade offer should exist");
            assert_eq!(trade_offer.status, String::from_str(&env, "Completed"));
        }

        // Verify all agreements exist
        for i in 0..all_agreement_ids.len() {
            let agreement_id = all_agreement_ids.get(i).unwrap();
            let barter_agreement = client
                .try_get_barter_agreement(&agreement_id)
                .unwrap()
                .expect("Barter agreement should exist");
            assert_eq!(barter_agreement.status, String::from_str(&env, "Active"));
        }

        // Update reputation for all cooperatives
        for i in 0..cooperatives.len() {
            let cooperative = cooperatives.get(i).unwrap();
            let result = client.try_update_reputation(&cooperative, &true);
            assert_is_success(result);
        }
    }

    #[test]
    fn test_trade_cancellation_and_reputation_impact() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        // Update reputation for failed trade (trade not completed)
        let result = client.try_update_reputation(&offering_cooperative, &false);
        assert_is_success(result);

        // Verify trade offer is still pending
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");
        assert_eq!(trade_offer.status, String::from_str(&env, "Pending"));

        // Later, someone accepts and completes the trade
        let accepting_cooperative = Address::generate(&env);
        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        let result = client.try_complete_trade(&offer_id, &offering_cooperative);
        assert_is_success(result);

        // Update reputation for successful trade
        let result = client.try_update_reputation(&offering_cooperative, &true);
        assert_is_success(result);

        // Verify trade is completed
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");
        assert_eq!(trade_offer.status, String::from_str(&env, "Completed"));
    }
}
