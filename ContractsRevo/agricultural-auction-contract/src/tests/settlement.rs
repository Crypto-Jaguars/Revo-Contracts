use crate::auction_core::AuctionOperations;
use crate::datatype::*;
use crate::tests::utils::*;
use crate::AgriculturalAuctionContract;
use soroban_sdk::testutils::Ledger;

#[test]
fn test_finalize_auction_success() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();

    // Create auction with short duration
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 100,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // Place bid
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    // Advance time past auction end
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = current_time + 200;
    });

    // Finalize auction
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_finalize_auction_not_found() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            999,
        )
    });

    assert_eq!(result, Err(AuctionError::AuctionNotFound));
}

#[test]
fn test_finalize_auction_not_yet_ended() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();

    // Create auction
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 3600,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // Place bid
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    // Try to finalize before auction ends
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
        )
    });

    assert_eq!(result, Err(AuctionError::AuctionNotYetEnded));
}

#[test]
fn test_finalize_auction_no_bids() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();

    // Create auction
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 100,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // Advance time past auction end
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = current_time + 200;
    });

    // Try to finalize without any bids
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
        )
    });

    assert_eq!(result, Err(AuctionError::NoBidsPlaced));
}

#[test]
fn test_finalize_auction_product_quantity_updates() {
    let test_env = setup_test();
    let mut product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();

    // Create auction
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 100,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // Place bid
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    // Advance time
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = current_time + 200;
    });

    // Finalize
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
        )
    });

    // Verify product quantity was updated
    product = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get(&DataKey::Product(test_env.farmer.clone(), 1))
            .unwrap()
    });

    assert_eq!(product.quantity, 0); // All quantity was in auction
}

#[test]
fn test_finalize_auction_highest_bidder_wins() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();

    // Create auction
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 100,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // Multiple bids
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2500,
            20,
            test_env.bidder2.clone(), // Highest bidder
            test_env.farmer.clone(),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2200,
            20,
            test_env.bidder3.clone(),
            test_env.farmer.clone(),
        )
    });

    // Advance time
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = current_time + 200;
    });

    // Finalize
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_finalize_auction_removes_auction() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();

    // Create auction
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 100,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // Place bid
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    // Advance time
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = current_time + 200;
    });

    // Finalize
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
        )
    });

    // Verify auction was removed
    let auction_exists = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .instance()
            .has(&DataKey::Auction(test_env.farmer.clone(), 1))
    });

    assert!(!auction_exists);
}

#[test]
fn test_auction_lifecycle_complete() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();

    // 1. Create auction
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 100,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // 2. Place multiple bids
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2500,
            20,
            test_env.bidder2.clone(),
            test_env.farmer.clone(),
        )
    });

    // 3. Extend auction
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::extend_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            current_time + 200,
        )
    });

    // 4. More bids after extension
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            3000,
            20,
            test_env.bidder3.clone(),
            test_env.farmer.clone(),
        )
    });

    // 5. Advance time
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = current_time + 300;
    });

    // 6. Finalize
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::finalize_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
        )
    });

    assert!(result.is_ok());
}
