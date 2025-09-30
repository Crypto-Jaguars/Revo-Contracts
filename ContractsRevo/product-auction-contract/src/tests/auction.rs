use crate::tests::utils::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::Address;

#[test]
fn test_create_auction_with_zero_reserve_price() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_test_product(&env, &client, &seller);
    let reserve_price = 0u64;
    let auction_end_time = env.ledger().timestamp() + 1000;

    client.create_auction(&seller, &reserve_price, &auction_end_time, &product_id);
    
    let auction = client.get_auction(&seller, &product_id);
    assert_eq!(auction.reserve_price, 0);
}

#[test]
fn test_create_multiple_auctions_different_products() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();
    setup_with_admin(&env, &client, true);

    let product_id_1 = create_test_product(&env, &client, &seller);
    let product_id_2 = create_test_product(&env, &client, &seller);

    let reserve_price = 50u64;
    let auction_end_time = env.ledger().timestamp() + 1000;

    client.create_auction(&seller, &reserve_price, &auction_end_time, &product_id_1);
    client.create_auction(&seller, &reserve_price, &auction_end_time, &product_id_2);

    let auction1 = client.get_auction(&seller, &product_id_1);
    let auction2 = client.get_auction(&seller, &product_id_2);

    assert_eq!(auction1.product_id, product_id_1);
    assert_eq!(auction2.product_id, product_id_2);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_get_nonexistent_auction() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    client.get_auction(&seller, &999u64);
}

#[test]
fn test_auction_expiry_boundary() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_test_product(&env, &client, &seller);
    let reserve_price = 50u64;
    let auction_end_time = env.ledger().timestamp() + 100;

    client.create_auction(&seller, &reserve_price, &auction_end_time, &product_id);

    env.ledger().set_timestamp(auction_end_time);
    
    let auction = client.get_auction(&seller, &product_id);
    assert_eq!(auction.auction_end_time, auction_end_time);
}