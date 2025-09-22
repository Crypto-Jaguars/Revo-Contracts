#![cfg(test)]

use super::*;
use crate::tests::utils::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

mod barter_agreement_creation {
    use super::*;

    #[test]
    fn test_create_barter_agreement_success() {
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

        // Accept trade offer (creates barter agreement)
        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

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
    fn test_create_multiple_barter_agreements() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let accepting_cooperative = Address::generate(&env);

        // Create multiple trade offers and accept them
        let mut agreement_ids = Vec::new(&env);
        for i in 0..3 {
            let offering_cooperative = Address::generate(&env);
            let offered_product =
                create_test_product(&env, &["product_a", "product_b", "product_c"][i % 3]);
            let requested_product =
                create_test_product(&env, &["requested_a", "requested_b", "requested_c"][i % 3]);

            let offer_id = client
                .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
                .unwrap()
                .expect("Trade offer creation should succeed");

            let agreement_id = client
                .try_accept_trade(&offer_id, &accepting_cooperative)
                .unwrap()
                .expect("Accept trade should succeed");

            agreement_ids.push_back(agreement_id);
        }

        // Verify all barter agreements exist and are unique
        assert_eq!(agreement_ids.len(), 3);
        for i in 0..agreement_ids.len() {
            for j in (i + 1)..agreement_ids.len() {
                let id1 = agreement_ids.get(i).unwrap();
                let id2 = agreement_ids.get(j).unwrap();
                assert_ne!(id1, id2, "Agreement IDs should be unique");
            }
        }

        // Verify each agreement can be retrieved
        for i in 0..agreement_ids.len() {
            let agreement_id = agreement_ids.get(i).unwrap();
            let barter_agreement = client
                .try_get_barter_agreement(&agreement_id)
                .unwrap()
                .expect("Barter agreement should exist");
            assert_eq!(barter_agreement.agreement_id, agreement_id);
        }
    }

    #[test]
    fn test_barter_agreement_unique_ids() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative1 = Address::generate(&env);
        let offering_cooperative2 = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);

        // Create two trade offers with same products but different cooperatives
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        let offer_id1 = client
            .try_create_trade_offer(&offering_cooperative1, &offered_product, &requested_product)
            .unwrap()
            .expect("First trade offer creation should succeed");

        let offer_id2 = client
            .try_create_trade_offer(&offering_cooperative2, &offered_product, &requested_product)
            .unwrap()
            .expect("Second trade offer creation should succeed");

        // Accept both offers
        let agreement_id1 = client
            .try_accept_trade(&offer_id1, &accepting_cooperative)
            .unwrap()
            .expect("First accept trade should succeed");

        let agreement_id2 = client
            .try_accept_trade(&offer_id2, &accepting_cooperative)
            .unwrap()
            .expect("Second accept trade should succeed");

        // Verify agreement IDs are unique
        assert_ne!(
            agreement_id1, agreement_id2,
            "Agreement IDs should be unique"
        );
        assert_ne!(offer_id1, offer_id2, "Offer IDs should be unique");
    }
}

mod barter_agreement_queries {
    use super::*;

    #[test]
    fn test_get_barter_agreement_success() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create and accept trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        // Get barter agreement details
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
    fn test_get_barter_agreement_not_found() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let nonexistent_agreement_id = create_test_product(&env, "nonexistent");

        let result = client.try_get_barter_agreement(&nonexistent_agreement_id);

        match result {
            Ok(inner_result) => {
                assert!(
                    inner_result.is_err(),
                    "Expected error for non-existent barter agreement"
                );
            }
            Err(_) => {
                // This is also acceptable - the error could cause the call to fail
            }
        }
    }

    #[test]
    fn test_barter_agreement_persistence() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create and accept trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        // Verify agreement persists across multiple queries
        for _ in 0..3 {
            let barter_agreement = client
                .try_get_barter_agreement(&agreement_id)
                .unwrap()
                .expect("Barter agreement should exist");
            assert_eq!(barter_agreement.agreement_id, agreement_id);
        }
    }
}

mod barter_agreement_validation {
    use super::*;

    #[test]
    fn test_barter_agreement_correct_parties() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create and accept trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        // Verify barter agreement has correct parties
        let barter_agreement = client
            .try_get_barter_agreement(&agreement_id)
            .unwrap()
            .expect("Barter agreement should exist");

        assert_eq!(barter_agreement.offering_cooperative, offering_cooperative);
        assert_eq!(
            barter_agreement.accepting_cooperative,
            accepting_cooperative
        );
        assert_eq!(barter_agreement.trade_offer_id, offer_id);
    }

    #[test]
    fn test_barter_agreement_status_consistency() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create and accept trade offer
        let offer_id = client
            .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        let agreement_id = client
            .try_accept_trade(&offer_id, &accepting_cooperative)
            .unwrap()
            .expect("Accept trade should succeed");

        // Verify both trade offer and barter agreement have consistent status
        let trade_offer = client
            .try_get_trade_details(&offer_id)
            .unwrap()
            .expect("Trade offer should exist");

        let barter_agreement = client
            .try_get_barter_agreement(&agreement_id)
            .unwrap()
            .expect("Barter agreement should exist");

        assert_eq!(trade_offer.status, String::from_str(&env, "Accepted"));
        assert_eq!(barter_agreement.status, String::from_str(&env, "Active"));
    }

    #[test]
    fn test_barter_agreement_after_trade_completion() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);
        let offered_product = create_test_product(&env, "corn");
        let requested_product = create_test_product(&env, "wheat");

        // Create complete trade flow
        let (offer_id, agreement_id) = create_complete_trade_flow(
            &env,
            &client,
            &offering_cooperative,
            &accepting_cooperative,
            &offered_product,
            &requested_product,
        );

        // Verify barter agreement still exists after completion
        let barter_agreement = client
            .try_get_barter_agreement(&agreement_id)
            .unwrap()
            .expect("Barter agreement should still exist");

        assert_eq!(barter_agreement.agreement_id, agreement_id);
        assert_eq!(barter_agreement.trade_offer_id, offer_id);
        assert_eq!(barter_agreement.status, String::from_str(&env, "Active"));
    }
}

mod barter_agreement_edge_cases {
    use super::*;

    #[test]
    fn test_barter_agreement_with_same_cooperative_different_offers() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let offering_cooperative = Address::generate(&env);
        let accepting_cooperative = Address::generate(&env);

        // Create two different trade offers from same cooperative
        let offered_product1 = create_test_product(&env, "corn");
        let requested_product1 = create_test_product(&env, "wheat");
        let offered_product2 = create_test_product(&env, "rice");
        let requested_product2 = create_test_product(&env, "beans");

        let offer_id1 = client
            .try_create_trade_offer(
                &offering_cooperative,
                &offered_product1,
                &requested_product1,
            )
            .unwrap()
            .expect("First trade offer creation should succeed");

        let offer_id2 = client
            .try_create_trade_offer(
                &offering_cooperative,
                &offered_product2,
                &requested_product2,
            )
            .unwrap()
            .expect("Second trade offer creation should succeed");

        // Accept both offers
        let agreement_id1 = client
            .try_accept_trade(&offer_id1, &accepting_cooperative)
            .unwrap()
            .expect("First accept trade should succeed");

        let agreement_id2 = client
            .try_accept_trade(&offer_id2, &accepting_cooperative)
            .unwrap()
            .expect("Second accept trade should succeed");

        // Verify both agreements exist and are different
        assert_ne!(agreement_id1, agreement_id2);
        assert_ne!(offer_id1, offer_id2);

        let barter_agreement1 = client
            .try_get_barter_agreement(&agreement_id1)
            .unwrap()
            .expect("First barter agreement should exist");

        let barter_agreement2 = client
            .try_get_barter_agreement(&agreement_id2)
            .unwrap()
            .expect("Second barter agreement should exist");

        assert_eq!(barter_agreement1.offering_cooperative, offering_cooperative);
        assert_eq!(barter_agreement2.offering_cooperative, offering_cooperative);
        assert_eq!(
            barter_agreement1.accepting_cooperative,
            accepting_cooperative
        );
        assert_eq!(
            barter_agreement2.accepting_cooperative,
            accepting_cooperative
        );
    }

    #[test]
    fn test_barter_agreement_with_different_accepting_cooperatives() {
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

        // Accept with first cooperative
        let agreement_id1 = client
            .try_accept_trade(&offer_id, &accepting_cooperative1)
            .unwrap()
            .expect("First accept trade should succeed");

        // Verify first agreement
        let barter_agreement1 = client
            .try_get_barter_agreement(&agreement_id1)
            .unwrap()
            .expect("First barter agreement should exist");

        assert_eq!(
            barter_agreement1.accepting_cooperative,
            accepting_cooperative1
        );

        // Try to accept with second cooperative (should fail)
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
    fn test_barter_agreement_id_uniqueness_across_multiple_trades() {
        let env = Env::default();
        let (_, client) = setup_contract_with_admin(&env);
        let accepting_cooperative = Address::generate(&env);

        let mut all_agreement_ids = Vec::new(&env);

        // Create multiple trade offers and accept them
        for i in 0..5 {
            let offering_cooperative = Address::generate(&env);
            let offered_product = create_test_product(
                &env,
                &[
                    "product_a",
                    "product_b",
                    "product_c",
                    "product_d",
                    "product_e",
                ][i % 5],
            );
            let requested_product = create_test_product(
                &env,
                &[
                    "requested_a",
                    "requested_b",
                    "requested_c",
                    "requested_d",
                    "requested_e",
                ][i % 5],
            );

            let offer_id = client
                .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
                .unwrap()
                .expect("Trade offer creation should succeed");

            let agreement_id = client
                .try_accept_trade(&offer_id, &accepting_cooperative)
                .unwrap()
                .expect("Accept trade should succeed");

            all_agreement_ids.push_back(agreement_id);
        }

        // Verify all agreement IDs are unique
        for i in 0..all_agreement_ids.len() {
            for j in (i + 1)..all_agreement_ids.len() {
                let id1 = all_agreement_ids.get(i).unwrap();
                let id2 = all_agreement_ids.get(j).unwrap();
                assert_ne!(id1, id2, "Agreement IDs should be unique across all trades");
            }
        }
    }
}
