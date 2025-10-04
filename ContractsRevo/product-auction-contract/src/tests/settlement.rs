use crate::tests::utils::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::Address;

#[test]
fn test_finalize_auction_updates_stock() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_auction_with_product(&env, &client, &seller, 100);

    let bidder = Address::generate(&env);
    client.place_bid(&product_id, &60u64, &bidder, &seller);

    env.ledger().set_timestamp(200);
    client.finalize_auction(&seller, &product_id);

    let product = client.get_product(&seller, &product_id);
    assert_eq!(product.stock, 9);
}

#[test]
fn test_finalize_auction_removes_auction() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_auction_with_product(&env, &client, &seller, 100);

    let bidder = Address::generate(&env);
    client.place_bid(&product_id, &60u64, &bidder, &seller);

    env.ledger().set_timestamp(200);
    client.finalize_auction(&seller, &product_id);

    let result = client.try_get_auction(&seller, &product_id);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_finalize_auction_nonexistent_product() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();
    setup_with_admin(&env, &client, true);

    let product_id = 999u64;
    let reserve_price = 50u64;
    let auction_end_time = env.ledger().timestamp() + 100;

    client.create_auction(&seller, &reserve_price, &auction_end_time, &product_id);

    let bidder = Address::generate(&env);
    client.place_bid(&product_id, &60u64, &bidder, &seller);

    env.ledger().set_timestamp(200);
    client.finalize_auction(&seller, &product_id);
}

#[test]
fn test_finalize_multiple_auctions() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id_1 = create_auction_with_product(&env, &client, &seller, 100);
    let product_id_2 = create_auction_with_product(&env, &client, &seller, 100);

    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    client.place_bid(&product_id_1, &60u64, &bidder1, &seller);
    client.place_bid(&product_id_2, &70u64, &bidder2, &seller);

    env.ledger().set_timestamp(200);

    client.finalize_auction(&seller, &product_id_1);
    client.finalize_auction(&seller, &product_id_2);

    let product1 = client.get_product(&seller, &product_id_1);
    let product2 = client.get_product(&seller, &product_id_2);

    assert_eq!(product1.stock, 9);
    assert_eq!(product2.stock, 9);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_finalize_with_single_stock_twice() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_test_product(&env, &client, &seller);

    client.update_stock(&seller, &product_id, &1u32);

    let reserve_price = 50u64;
    let auction_end_time_1 = env.ledger().timestamp() + 100;
    let auction_end_time_2 = env.ledger().timestamp() + 200;

    client.create_auction(&seller, &reserve_price, &auction_end_time_1, &product_id);

    let bidder1 = Address::generate(&env);
    client.place_bid(&product_id, &60u64, &bidder1, &seller);

    env.ledger().set_timestamp(150);
    client.finalize_auction(&seller, &product_id);

    client.create_auction(&seller, &reserve_price, &auction_end_time_2, &product_id);

    let bidder2 = Address::generate(&env);
    client.place_bid(&product_id, &70u64, &bidder2, &seller);

    env.ledger().set_timestamp(250);
    client.finalize_auction(&seller, &product_id);
}
