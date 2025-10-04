use crate::auction_core::AuctionOperations;
use crate::datatype::*;
use crate::tests::utils::*;
use crate::AgriculturalAuctionContract;
use soroban_sdk::testutils::Ledger;

#[test]
fn test_create_auction_success() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    // Save product first
    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let current_time = test_env.env.ledger().timestamp();
    let result = test_env.env.as_contract(&test_env.contract_id, || {
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

    assert!(result.is_ok());
}

#[test]
fn test_create_auction_duplicate() {
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

    // Create first auction
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

    // Try to create duplicate
    let result = test_env.env.as_contract(&test_env.contract_id, || {
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

    assert_eq!(result, Err(AuctionError::AuctionAlreadyExists));
}

#[test]
fn test_create_auction_product_not_found() {
    let test_env = setup_test();
    let current_time = test_env.env.ledger().timestamp();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            999,
            STANDARD_RESERVE_PRICE,
            current_time + 3600,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    assert_eq!(result, Err(AuctionError::ProductNotFound));
}

#[test]
fn test_create_auction_product_expired() {
    let test_env = setup_test();

    // Set ledger to a future time
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = 100000;
    });

    let current_time = test_env.env.ledger().timestamp();

    let mut product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);
    product.expiry_date = current_time - 1; // Already expired

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let result = test_env.env.as_contract(&test_env.contract_id, || {
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

    assert_eq!(result, Err(AuctionError::ProductExpired));
}

#[test]
fn test_create_auction_invalid_end_time_past() {
    let test_env = setup_test();

    // Set ledger to a future time
    test_env.env.ledger().with_mut(|li| {
        li.timestamp = 100000;
    });

    let current_time = test_env.env.ledger().timestamp();

    let mut product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);
    product.expiry_date = current_time + 86400; // Valid expiry

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time - 100, // Past time
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    assert_eq!(result, Err(AuctionError::InvalidAuctionEndTime));
}

#[test]
fn test_create_auction_invalid_end_time_after_expiry() {
    let test_env = setup_test();
    let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);

    test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .set(&DataKey::Product(test_env.farmer.clone(), 1), &product);
    });

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            product.expiry_date + 100, // After expiry
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    assert_eq!(result, Err(AuctionError::InvalidAuctionEndTime));
}

#[test]
fn test_create_auction_zero_min_quantity() {
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
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 3600,
            0, // Zero min quantity
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    assert_eq!(result, Err(AuctionError::QuantityUnavailable));
}

#[test]
fn test_create_auction_min_quantity_exceeds_available() {
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
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 3600,
            product.quantity + 1, // More than available
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            false,
        )
    });

    assert_eq!(result, Err(AuctionError::QuantityUnavailable));
}

#[test]
fn test_create_auction_with_dynamic_pricing() {
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
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::create_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            STANDARD_RESERVE_PRICE,
            current_time + 3600,
            STANDARD_MIN_QUANTITY,
            STANDARD_BULK_THRESHOLD,
            STANDARD_BULK_DISCOUNT,
            true, // Dynamic pricing enabled
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_extend_auction_success() {
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

    // Extend auction
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::extend_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            current_time + 7200, // 2 hours later
        )
    });

    assert!(result.is_ok());
}

#[test]
fn test_extend_auction_not_found() {
    let test_env = setup_test();

    let current_time = test_env.env.ledger().timestamp();
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::extend_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            999,
            current_time + 7200,
        )
    });

    assert_eq!(result, Err(AuctionError::AuctionNotFound));
}

#[test]
fn test_extend_auction_invalid_new_time() {
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

    // Try to extend to earlier time
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <AgriculturalAuctionContract as AuctionOperations>::extend_auction(
            test_env.env.clone(),
            test_env.farmer.clone(),
            1,
            current_time + 1800, // Earlier than current end time
        )
    });

    assert_eq!(result, Err(AuctionError::InvalidAuctionEndTime));
}

#[test]
fn test_create_multiple_auctions() {
    let test_env = setup_test();

    for i in 1..=3 {
        let product = create_standard_product(&test_env.env, test_env.farmer.clone(), i);
        test_env.env.as_contract(&test_env.contract_id, || {
            test_env
                .env
                .storage()
                .persistent()
                .set(&DataKey::Product(test_env.farmer.clone(), i), &product);
        });

        let current_time = test_env.env.ledger().timestamp();
        let result = test_env.env.as_contract(&test_env.contract_id, || {
            <AgriculturalAuctionContract as AuctionOperations>::create_auction(
                test_env.env.clone(),
                test_env.farmer.clone(),
                i,
                STANDARD_RESERVE_PRICE,
                current_time + 3600,
                STANDARD_MIN_QUANTITY,
                STANDARD_BULK_THRESHOLD,
                STANDARD_BULK_DISCOUNT,
                false,
            )
        });

        assert!(result.is_ok(), "Auction {} creation failed", i);
    }
}
