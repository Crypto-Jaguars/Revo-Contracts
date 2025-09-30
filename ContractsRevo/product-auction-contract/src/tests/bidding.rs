use crate::tests::utils::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::Address;

#[test]
fn test_place_multiple_bids_increasing() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();
    setup_with_admin(&env, &client, true);

    let product_id = create_auction_with_product(&env, &client, &seller, 1000);

    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    let bidder3 = Address::generate(&env);

    client.place_bid(&product_id, &60u64, &bidder1, &seller);
    client.place_bid(&product_id, &80u64, &bidder2, &seller);
    client.place_bid(&product_id, &100u64, &bidder3, &seller);

    let auction = client.get_auction(&seller, &product_id);
    assert_eq!(auction.highest_bid, 100);
    assert_eq!(auction.highest_bidder, Some(bidder3));
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_bid_equal_to_current_highest() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_auction_with_product(&env, &client, &seller, 1000);

    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    client.place_bid(&product_id, &100u64, &bidder1, &seller);
    client.place_bid(&product_id, &100u64, &bidder2, &seller);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_bid_below_reserve_price() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_auction_with_product(&env, &client, &seller, 1000);
    let bidder = Address::generate(&env);

    client.place_bid(&product_id, &30u64, &bidder, &seller);
}

#[test]
fn test_same_bidder_increases_bid() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_auction_with_product(&env, &client, &seller, 1000);
    let bidder = Address::generate(&env);

    client.place_bid(&product_id, &60u64, &bidder, &seller);
    client.place_bid(&product_id, &80u64, &bidder, &seller);

    let auction = client.get_auction(&seller, &product_id);
    assert_eq!(auction.highest_bid, 80);
    assert_eq!(auction.highest_bidder, Some(bidder));
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_bid_after_auction_ends() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_auction_with_product(&env, &client, &seller, 100);
    
    env.ledger().set_timestamp(200);
    
    let bidder = Address::generate(&env);
    client.place_bid(&product_id, &60u64, &bidder, &seller);
}

#[test]
fn test_bid_exactly_at_reserve_price() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_auction_with_product(&env, &client, &seller, 1000);
    let bidder = Address::generate(&env);

    client.place_bid(&product_id, &50u64, &bidder, &seller);

    let auction = client.get_auction(&seller, &product_id);
    assert_eq!(auction.highest_bid, 50);
}