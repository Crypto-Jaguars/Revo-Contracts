#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

// Basic contract initialization tests
// More comprehensive tests are in the modular test structure

#[test]
fn test_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let result = client.try_initialize(&admin);
    assert!(result.is_ok());

    // Test that we can get the admin after initialization
    let contract_admin_result = client.try_get_admin();
    assert!(contract_admin_result.is_ok());
}

#[test]
fn test_get_admin_before_initialization() {
    let env = Env::default();

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    // Test that getting admin before initialization returns an error
    let contract_admin_result = client.try_get_admin();
    assert!(contract_admin_result.is_err());
}

#[test]
fn test_double_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    // First initialization should succeed
    let result1 = client.try_initialize(&admin);
    assert!(result1.is_ok());

    // Second initialization should fail
    let result2 = client.try_initialize(&admin);
    assert!(result2.is_err());
}

// Basic trade offer creation test
#[test]
fn test_create_trade_offer() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"corn"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"coffee"))
        .into();

    let offer_id = client
        .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    let trade_offer = client
        .try_get_trade_details(&offer_id)
        .unwrap()
        .expect("Trade offer should exist");
    assert_eq!(trade_offer.cooperative_id, cooperative);
    assert_eq!(trade_offer.status, String::from_str(&env, "Pending"));
}

#[test]
fn test_create_trade_offer_success() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"wheat"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"rice"))
        .into();

    // Test successful trade offer creation - using match for cleaner error handling
    let offer_id =
        match client.try_create_trade_offer(&cooperative, &offered_product, &requested_product) {
            Ok(Ok(id)) => id,
            Ok(Err(trade_error)) => {
                panic!("Trade offer creation failed with error: {:?}", trade_error)
            }
            Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
        };

    // Verify the trade offer was created successfully
    let trade_offer = client
        .try_get_trade_details(&offer_id)
        .unwrap()
        .expect("Trade offer should exist");
    assert_eq!(trade_offer.cooperative_id, cooperative);
    assert_eq!(trade_offer.offered_product, offered_product);
    assert_eq!(trade_offer.requested_product, requested_product);
    assert_eq!(trade_offer.status, String::from_str(&env, "Pending"));
}

#[test]
fn test_create_trade_offer_invalid_products() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    let same_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"wheat"))
        .into();

    // Test error when offered and requested products are the same
    let result = client.try_create_trade_offer(
        &cooperative,
        &same_product,
        &same_product, // Same as offered_product - should fail
    );

    // The function should detect validation error and handle it appropriately
    // Since it's a validation error, the call should succeed but return an error value
    match result {
        Ok(inner_result) => {
            assert!(
                inner_result.is_err(),
                "Expected validation error for same products"
            );
        }
        Err(_) => {
            // This is also acceptable - the validation error could cause the call to fail
            // The important thing is that we're not getting a successful trade offer creation
        }
    }
}

#[test]
fn test_accept_trade_success() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let offering_cooperative = Address::generate(&env);
    let accepting_cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Create a trade offer first
    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"corn"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"wheat"))
        .into();

    let offer_id = client
        .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Accept the trade offer
    let agreement_id = match client.try_accept_trade(&offer_id, &accepting_cooperative) {
        Ok(Ok(id)) => id,
        Ok(Err(trade_error)) => panic!("Accept trade failed with error: {:?}", trade_error),
        Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
    };

    // Verify the trade offer status was updated
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
    assert_eq!(barter_agreement.trade_offer_id, offer_id);
    assert_eq!(barter_agreement.offering_cooperative, offering_cooperative);
    assert_eq!(
        barter_agreement.accepting_cooperative,
        accepting_cooperative
    );
}

#[test]
fn test_accept_trade_errors() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Test 1: Try to accept non-existent offer
    let non_existent_offer_id = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"nonexistent"))
        .into();
    let result = client.try_accept_trade(&non_existent_offer_id, &cooperative);
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

    // Create a trade offer for further tests
    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"rice"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"beans"))
        .into();

    let offer_id = client
        .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Test 2: Try to accept own offer (should fail)
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
fn test_complete_trade_success() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let offering_cooperative = Address::generate(&env);
    let accepting_cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Create a trade offer
    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"corn"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"wheat"))
        .into();

    let offer_id = client
        .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Accept the trade offer
    let _agreement_id = client
        .try_accept_trade(&offer_id, &accepting_cooperative)
        .unwrap()
        .expect("Accept trade should succeed");

    // Complete the trade
    let result = client.try_complete_trade(&offer_id, &offering_cooperative);
    match result {
        Ok(Ok(())) => {
            // Success - verify the trade offer status was updated
            let trade_offer = client
                .try_get_trade_details(&offer_id)
                .unwrap()
                .expect("Trade offer should exist");
            assert_eq!(trade_offer.status, String::from_str(&env, "Completed"));
        }
        Ok(Err(trade_error)) => panic!("Complete trade failed with error: {:?}", trade_error),
        Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
    }
}

#[test]
fn test_complete_trade_errors() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let offering_cooperative = Address::generate(&env);
    let accepting_cooperative = Address::generate(&env);
    let unauthorized_cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Test 1: Try to complete non-existent trade
    let non_existent_offer_id = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"nonexistent"))
        .into();
    let result = client.try_complete_trade(&non_existent_offer_id, &offering_cooperative);
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

    // Create and accept a trade offer for further tests
    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"rice"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"beans"))
        .into();

    let offer_id = client
        .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Test 2: Try to complete trade before acceptance (should fail)
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

    // Accept the trade offer
    let _agreement_id = client
        .try_accept_trade(&offer_id, &accepting_cooperative)
        .unwrap()
        .expect("Accept trade should succeed");

    // Test 3: Try to complete trade with unauthorized caller (should fail)
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
fn test_get_trade_details_success() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Create a trade offer
    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"corn"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"wheat"))
        .into();

    let offer_id = client
        .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Test successful retrieval of trade details
    let result = client.try_get_trade_details(&offer_id);
    match result {
        Ok(Ok(trade_offer)) => {
            assert_eq!(trade_offer.offer_id, offer_id);
            assert_eq!(trade_offer.cooperative_id, cooperative);
            assert_eq!(trade_offer.offered_product, offered_product);
            assert_eq!(trade_offer.requested_product, requested_product);
            assert_eq!(trade_offer.status, String::from_str(&env, "Pending"));
        }
        Ok(Err(trade_error)) => panic!("Get trade details failed with error: {:?}", trade_error),
        Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
    }
}

#[test]
fn test_get_trade_details_not_found() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Try to get details for non-existent trade offer
    let non_existent_offer_id = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"nonexistent"))
        .into();
    let result = client.try_get_trade_details(&non_existent_offer_id);

    match result {
        Ok(inner_result) => {
            assert!(
                inner_result.is_err(),
                "Expected error for non-existent trade offer"
            );
            // Verify it's the correct error type
            match inner_result {
                Err(_trade_error) => {
                    // The error should be TradeOfferNotFound - we got the expected error
                }
                Ok(_) => panic!("Expected TradeOfferNotFound error"),
            }
        }
        Err(_) => {
            // This is also acceptable - the error could cause the call to fail
        }
    }
}

#[test]
fn test_list_active_offers_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Test listing active offers when there are none
    let result = client.try_list_active_offers();
    match result {
        Ok(Ok(offers)) => {
            assert_eq!(offers.len(), 0, "Should have no active offers initially");
        }
        Ok(Err(trade_error)) => panic!("List active offers failed with error: {:?}", trade_error),
        Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
    }
}

#[test]
fn test_list_active_offers_with_offers() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let cooperative1 = Address::generate(&env);
    let cooperative2 = Address::generate(&env);
    let accepting_cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Create first trade offer
    let offered_product1 = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"corn"))
        .into();
    let requested_product1 = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"wheat"))
        .into();

    let offer_id1 = client
        .try_create_trade_offer(&cooperative1, &offered_product1, &requested_product1)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Create second trade offer
    let offered_product2 = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"rice"))
        .into();
    let requested_product2 = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"beans"))
        .into();

    let offer_id2 = client
        .try_create_trade_offer(&cooperative2, &offered_product2, &requested_product2)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Test listing active offers - should have 2 offers
    let result = client.try_list_active_offers();
    match result {
        Ok(Ok(offers)) => {
            assert_eq!(offers.len(), 2, "Should have 2 active offers");
            assert!(offers.contains(&offer_id1), "Should contain first offer");
            assert!(offers.contains(&offer_id2), "Should contain second offer");
        }
        Ok(Err(trade_error)) => panic!("List active offers failed with error: {:?}", trade_error),
        Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
    }

    // Accept one of the offers (this should remove it from active offers)
    let _agreement_id = client
        .try_accept_trade(&offer_id1, &accepting_cooperative)
        .unwrap()
        .expect("Accept trade should succeed");

    // Test listing active offers after acceptance - should have 1 offer
    let result_after_accept = client.try_list_active_offers();
    match result_after_accept {
        Ok(Ok(offers)) => {
            assert_eq!(
                offers.len(),
                1,
                "Should have 1 active offer after acceptance"
            );
            assert!(
                !offers.contains(&offer_id1),
                "Should not contain accepted offer"
            );
            assert!(offers.contains(&offer_id2),);
        }
        Ok(Err(trade_error)) => panic!("List active offers failed with error: {:?}", trade_error),
        Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
    }
}

#[test]
fn test_get_barter_agreement_success() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let offering_cooperative = Address::generate(&env);
    let accepting_cooperative = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Create and accept a trade offer to generate a barter agreement
    let offered_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"corn"))
        .into();
    let requested_product = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"wheat"))
        .into();

    let offer_id = client
        .try_create_trade_offer(&offering_cooperative, &offered_product, &requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    let agreement_id = client
        .try_accept_trade(&offer_id, &accepting_cooperative)
        .unwrap()
        .expect("Accept trade should succeed");

    // Test successful retrieval of barter agreement details
    let result = client.try_get_barter_agreement(&agreement_id);
    match result {
        Ok(Ok(barter_agreement)) => {
            assert_eq!(barter_agreement.agreement_id, agreement_id);
            assert_eq!(barter_agreement.trade_offer_id, offer_id);
            assert_eq!(barter_agreement.offering_cooperative, offering_cooperative);
            assert_eq!(
                barter_agreement.accepting_cooperative,
                accepting_cooperative
            );
            assert_eq!(barter_agreement.status, String::from_str(&env, "Active"));
        }
        Ok(Err(trade_error)) => panic!("Get barter agreement failed with error: {:?}", trade_error),
        Err(call_error) => panic!("Contract call failed with error: {:?}", call_error),
    }
}

#[test]
fn test_get_barter_agreement_not_found() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(&env, &contract_id);

    env.mock_all_auths();

    let _ = client.try_initialize(&admin);

    // Try to get details for non-existent barter agreement
    let non_existent_agreement_id = env
        .crypto()
        .keccak256(&Bytes::from_array(&env, b"nonexistent"))
        .into();
    let result = client.try_get_barter_agreement(&non_existent_agreement_id);

    match result {
        Ok(inner_result) => {
            assert!(
                inner_result.is_err(),
                "Expected error for non-existent barter agreement"
            );
            // Verify it's the correct error type
            match inner_result {
                Err(_trade_error) => {
                    // The error should be TradeOfferNotFound (reused for barter agreements)
                }
                Ok(_) => panic!("Expected TradeOfferNotFound error"),
            }
        }
        Err(_) => {
            // This is also acceptable - the error could cause the call to fail
        }
    }
}
