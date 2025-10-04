use crate::auction_core::AuctionOperations;
use crate::datatype::*;
use crate::tests::utils::*;
use crate::AgriculturalAuctionContract;
use soroban_sdk::testutils::Address as _;

#[test]
fn test_place_bid_success() {
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
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000, // bid amount
            20,   // quantity
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_place_bid_farmer_cannot_bid() {
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

    // Farmer tries to bid on own auction
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2000,
            20,
            test_env.farmer.clone(), // Farmer as bidder
            test_env.farmer.clone(),
        )
    });

    assert_eq!(result, Err(AuctionError::InvalidBidder));
}

#[test]
fn test_place_bid_auction_not_found() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            999,
            2000,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    assert_eq!(result, Err(AuctionError::AuctionNotFound));
}

#[test]
fn test_place_bid_too_low() {
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

    // Place bid below reserve price
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            500, // Too low (50 per unit, reserve is 100)
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    assert_eq!(result, Err(AuctionError::BidTooLow));
}

#[test]
fn test_place_bid_quantity_below_minimum() {
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
            STANDARD_MIN_QUANTITY, // 10
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    // Place bid with quantity below minimum
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            500,
            5, // Below minimum of 10
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    assert_eq!(result, Err(AuctionError::QuantityUnavailable));
}

#[test]
fn test_place_bid_quantity_exceeds_available() {
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

    // Place bid with quantity exceeding available
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            20000,
            product.quantity + 1, // More than available
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    assert_eq!(result, Err(AuctionError::QuantityUnavailable));
}

#[test]
fn test_place_multiple_bids_same_bidder() {
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

    // First bid
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

    // Second higher bid from same bidder
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            2500,
            20,
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_place_bid_competitive_bidding() {
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

    // Bidder 1
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

    // Bidder 2 with higher bid
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

    // Bidder 3 with highest bid
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            3000,
            20,
            test_env.bidder3.clone(),
            test_env.farmer.clone(),
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_place_bid_with_bulk_discount() {
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
            STANDARD_BULK_THRESHOLD, // 50
            STANDARD_BULK_DISCOUNT,  // 10%
            false,
        )
    });

    // Place bid meeting bulk threshold
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::place_bid(
            test_env.env.clone(),
            1,
            10000,
            50, // Exactly at threshold
            test_env.bidder1.clone(),
            test_env.farmer.clone(),
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_high_volume_bidding() {
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

    // Place 20 bids
    for i in 1..=20 {
        let bidder = soroban_sdk::Address::generate(&test_env.env);
        let result = test_env.env.as_contract(&test_env.contract_id, || {
            <AgriculturalAuctionContract as AuctionOperations>::place_bid(
                test_env.env.clone(),
                1,
                2000 + (i * 100),
                20,
                bidder.clone(),
                test_env.farmer.clone(),
            )
        });

        assert!(result.is_ok(), "Bid {} failed", i);
    }
}
