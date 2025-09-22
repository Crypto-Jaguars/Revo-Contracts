#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, String};

/// Test helper to create a new contract instance with admin
pub fn setup_contract_with_admin(env: &Env) -> (Address, CrossCooperativeTradeContractClient) {
    let admin = Address::generate(env);
    let contract_id = env.register(CrossCooperativeTradeContract, ());
    let client = CrossCooperativeTradeContractClient::new(env, &contract_id);

    env.mock_all_auths();
    let _ = client.try_initialize(&admin).unwrap();

    (admin, client)
}

/// Test helper to create a new contract instance without initialization
pub fn setup_contract(env: &Env) -> CrossCooperativeTradeContractClient {
    let contract_id = env.register(CrossCooperativeTradeContract, ());
    CrossCooperativeTradeContractClient::new(env, &contract_id)
}

/// Generate a test product token hash
pub fn create_test_product(env: &Env, product_name: &str) -> BytesN<32> {
    let bytes = product_name.as_bytes();
    let mut array = [0u8; 32];
    let len = bytes.len().min(32);
    array[..len].copy_from_slice(&bytes[..len]);
    env.crypto()
        .keccak256(&Bytes::from_array(env, &array))
        .into()
}

/// Create multiple test cooperatives
pub fn create_test_cooperatives(env: &Env, count: usize) -> Vec<Address> {
    let mut cooperatives = Vec::new(env);
    for _ in 0..count {
        cooperatives.push_back(Address::generate(env));
    }
    cooperatives
}

/// Create a complete trade flow: offer -> accept -> complete
pub fn create_complete_trade_flow(
    env: &Env,
    client: &CrossCooperativeTradeContractClient,
    offering_coop: &Address,
    accepting_coop: &Address,
    offered_product: &BytesN<32>,
    requested_product: &BytesN<32>,
) -> (BytesN<32>, BytesN<32>) {
    // Create trade offer
    let offer_id = client
        .try_create_trade_offer(offering_coop, offered_product, requested_product)
        .unwrap()
        .expect("Trade offer creation should succeed");

    // Accept trade offer
    let agreement_id = client
        .try_accept_trade(&offer_id, accepting_coop)
        .unwrap()
        .expect("Accept trade should succeed");

    // Complete trade
    let _ = client
        .try_complete_trade(&offer_id, offering_coop)
        .unwrap()
        .expect("Complete trade should succeed");

    (offer_id, agreement_id)
}

/// Verify trade offer details match expected values
pub fn assert_trade_offer_matches(
    trade_offer: &TradeOffer,
    expected_offer_id: &BytesN<32>,
    expected_cooperative: &Address,
    expected_offered_product: &BytesN<32>,
    expected_requested_product: &BytesN<32>,
    expected_status: &str,
    env: &Env,
) {
    assert_eq!(trade_offer.offer_id, *expected_offer_id);
    assert_eq!(trade_offer.cooperative_id, *expected_cooperative);
    assert_eq!(trade_offer.offered_product, *expected_offered_product);
    assert_eq!(trade_offer.requested_product, *expected_requested_product);
    assert_eq!(trade_offer.status, String::from_str(env, expected_status));
}

/// Verify barter agreement details match expected values
pub fn assert_barter_agreement_matches(
    agreement: &BarterAgreement,
    expected_agreement_id: &BytesN<32>,
    expected_trade_offer_id: &BytesN<32>,
    expected_offering_coop: &Address,
    expected_accepting_coop: &Address,
    expected_status: &str,
    env: &Env,
) {
    assert_eq!(agreement.agreement_id, *expected_agreement_id);
    assert_eq!(agreement.trade_offer_id, *expected_trade_offer_id);
    assert_eq!(agreement.offering_cooperative, *expected_offering_coop);
    assert_eq!(agreement.accepting_cooperative, *expected_accepting_coop);
    assert_eq!(agreement.status, String::from_str(env, expected_status));
}

/// Verify reputation details match expected values
pub fn assert_reputation_matches(
    reputation: &Reputation,
    expected_cooperative: &Address,
    expected_successful_trades: u32,
    expected_rating: u32,
) {
    assert_eq!(reputation.cooperative_id, *expected_cooperative);
    assert_eq!(reputation.successful_trades, expected_successful_trades);
    assert_eq!(reputation.rating, expected_rating);
}

/// Create a test environment with multiple active trade offers
pub fn setup_multiple_trade_offers(
    env: &Env,
    client: &CrossCooperativeTradeContractClient,
    num_offers: usize,
) -> Vec<(BytesN<32>, Address, BytesN<32>, BytesN<32>)> {
    let mut offers = Vec::new(env);

    for i in 0..num_offers {
        let cooperative = Address::generate(env);
        // Create unique product names using simple patterns
        let offered_product = create_test_product(
            env,
            &[
                "product_a",
                "product_b",
                "product_c",
                "product_d",
                "product_e",
            ][i % 5],
        );
        let requested_product = create_test_product(
            env,
            &[
                "requested_a",
                "requested_b",
                "requested_c",
                "requested_d",
                "requested_e",
            ][i % 5],
        );

        let offer_id = client
            .try_create_trade_offer(&cooperative, &offered_product, &requested_product)
            .unwrap()
            .expect("Trade offer creation should succeed");

        offers.push_back((offer_id, cooperative, offered_product, requested_product));
    }

    offers
}

/// Verify that an error result contains the expected error type
pub fn assert_error_contains<T, E: PartialEq + core::fmt::Debug>(
    result: Result<T, E>,
    expected_error: E,
) {
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(error) => assert_eq!(error, expected_error),
    }
}

/// Verify that a result is an error (without checking specific error type)
pub fn assert_is_error<T, E>(result: Result<T, E>) {
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(_) => {} // Expected
    }
}

/// Verify that a result is successful (without checking specific value)
pub fn assert_is_success<T, E: core::fmt::Debug>(result: Result<T, E>) {
    match result {
        Ok(_) => {} // Expected
        Err(error) => panic!("Expected success but got error: {:?}", error),
    }
}
