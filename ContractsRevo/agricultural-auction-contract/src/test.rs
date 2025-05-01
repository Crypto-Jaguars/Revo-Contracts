#![cfg(test)]

extern crate std; // Needed for vec! macro in tests sometimes

// Ensure ProductDetails is correctly imported if it's in a submodule
use crate::product_listing::ProductDetails;
// Import necessary types from the main lib
use crate::{
    AdminError, AgriculturalAuctionContract, AgriculturalAuctionContractClient, AuctionError,
    DataKey, FreshnessRating, MarketPrice, OracleError, ProductError, QualityGrade, SeasonalStatus,
    StorageCondition, TimeError,
};

use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _}, // Import test utilities
    vec,                                                 // Soroban Vec
    Address,
    Env,
    Error, // Soroban Error enum
    IntoVal,
    String, // Soroban String
    Symbol,
    Val, // Soroban Val
    Vec,
};

// Helper function to set up the test environment
fn setup_test<'a>() -> (
    Env,
    Address, // Contract ID
    AgriculturalAuctionContractClient<'a>,
    Address, // Admin
    Address, // Farmer 1
    Address, // Farmer 2
    Address, // Bidder 1
    Address, // Bidder 2
) {
    let env = Env::default();
    env.mock_all_auths(); // Automatically approve all auth calls for convenience

    // Generate identities
    let admin = Address::generate(&env);
    let farmer1 = Address::generate(&env);
    let farmer2 = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);

    // Register the contract
    // Use register_contract for contracts, not register() which is for custom types
    let contract_id = env.register_contract(None, AgriculturalAuctionContract);
    let client = AgriculturalAuctionContractClient::new(&env, &contract_id);

    // Initialize the contract
    client.initialize(&admin);

    (
        env,
        contract_id, // Return the contract Address (ID)
        client,
        admin,
        farmer1,
        farmer2,
        bidder1,
        bidder2,
    )
}

// Helper function to create default product details
fn create_product_details(env: &Env, name: &str, p_type: &str, region: &str) -> ProductDetails {
    ProductDetails {
        name: Symbol::new(env, name),
        description: String::from_slice(env, "A test product description."), // Use from_slice
        base_price: 1000,                                                    // e.g., 10.00 units
        weight_kg: 50,
        quantity: 100,
        harvest_date: env.ledger().timestamp(), // Harvested now
        images: vec![&env, String::from_slice(env, "img1.url")], // Use from_slice
        certifications: vec![&env, Symbol::new(env, "Organic")],
        storage_condition: StorageCondition::RoomTemperature,
        product_type: Symbol::new(env, p_type),
        region: Symbol::new(env, region),
    }
}

// Helper to advance ledger time
fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|li| {
        li.timestamp += seconds;
    });
}

const DAY_SECS: u64 = 24 * 60 * 60;

// Initialization and Admin Tests

#[test]
fn test_initialize_contract() {
    let (env, _, client, admin, _, _, _, _) = setup_test();

    // Check if admin is set correctly using the non-try method
    assert_eq!(client.get_admin(), admin.clone()); // get_admin returns Result, check inner value

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        Symbol::new(&env, "contract_initialized").into_val(&env), // Convert Symbols to Val for topics
        admin.into_val(&env), // Convert Address to Val for topics
    ];
    match event {
        (_, topics, _) if topics == expected_topics => (), // Compare Vec<Val>
        _ => panic!(
            "Event does not match expected format. Topics: {:?}",
            event.1
        ),
    }
}

#[test]
fn test_initialize_already_initialized() {
    let (_, _, client, admin, _, _, _, _) = setup_test();
    let result = client.try_initialize(&admin); // Use try_ method for expected errors

    // Correctly match the nested Result structure for ContractError
    match result {
        Err(Ok(e)) if (e) == AdminError::AlreadyInitialized => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

// Product Listing Tests

#[test]
fn test_add_product_success() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Tomato", "Fruit", "North");

    // Add product - non-try method returns u64 or panics
    let prod_id = client.add_product(&farmer1, &details);
    // No need for is_ok() check, if it failed, it would have panicked

    // Verify using get_product
    let product = client.get_product(&farmer1, &prod_id);
    assert_eq!(product.id, prod_id);
    assert_eq!(product.farmer, farmer1);
    assert_eq!(product.name, details.name);
    assert_eq!(product.base_price, details.base_price);
    assert_eq!(product.current_price, details.base_price); // Initial price
    assert_eq!(product.quantity, details.quantity);
    assert_eq!(product.harvest_date, details.harvest_date);
    assert_eq!(product.quality_grade, QualityGrade::GradeB); // Default
    assert_eq!(product.freshness_rating, FreshnessRating::Premium); // Harvested now

    // Verify using get_products (list)
    let products = client.get_products(&farmer1);
    assert_eq!(products.len(), 1);
    assert_eq!(products.get_unchecked(0).id, prod_id); // Use get_unchecked for direct access after len check

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        farmer1.into_val(&env),                           // Convert Address
        Symbol::new(&env, "ProductAdded").into_val(&env), // Convert Symbol
        details.name.into_val(&env),                      // Convert Symbol
    ];
    match event {
        (
            _, // Contract ID
            topics,
            _, // Data (product clone)
        ) if topics == expected_topics => (),
        _ => panic!(
            "Event does not match expected format. Topics: {:?}",
            event.1
        ),
    }
}

#[test]
fn test_add_product_invalid_description() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let mut details = create_product_details(&env, "Tomato", "Fruit", "North");
    details.description = String::from_slice(&env, "Too short"); // Use from_slice

    let result = client.try_add_product(&farmer1, &details);
    match result {
        Err(Ok(e)) if (e) == ProductError::InvalidDescription => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }

    details.description = String::from_slice(&env, &"a".repeat(501)); // Use from_slice
    let result = client.try_add_product(&farmer1, &details);
    match result {
        Err(Ok(e)) if (e) == ProductError::InvalidDescription => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_add_product_invalid_price() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let mut details = create_product_details(&env, "Tomato", "Fruit", "North");
    details.base_price = 0;
    let result = client.try_add_product(&farmer1, &details);
    match result {
        Err(Ok(e)) if (e) == ProductError::InvalidPrice => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_add_product_invalid_images() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let mut details = create_product_details(&env, "Tomato", "Fruit", "North");
    details.images = vec![&env]; // Empty vec

    let result = client.try_add_product(&farmer1, &details);
    match result {
        Err(Ok(e)) if (e) == ProductError::InvalidImageCount => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_add_product_invalid_harvest_date() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let mut details = create_product_details(&env, "Tomato", "Fruit", "North");
    details.harvest_date = env.ledger().timestamp() + 1000; // In the future
    let result = client.try_add_product(&farmer1, &details);
    match result {
        Err(Ok(e)) if (e) == ProductError::InvalidHarvestDate => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_get_nonexistent_product() {
    let (_, _, client, _, farmer1, _, _, _) = setup_test();
    let result = client.try_get_product(&farmer1, &123); // Non-existent ID
    match result {
        Err(Ok(e)) if (e) == ProductError::ProductNotFound => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_update_quantity_success() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Carrot", "Root_Vegetables", "South");
    let product_id = client.add_product(&farmer1, &details);

    let new_quantity = 50u32;
    // update_quantity returns Result<(), ProductError>, so non-try version panics or returns ()
    client.update_quantity(&farmer1, &product_id, &new_quantity);
    // No .is_ok() needed

    let product = client.get_product(&farmer1, &product_id);
    assert_eq!(product.quantity, new_quantity);

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        farmer1.into_val(&env),                              // Convert Address
        Symbol::new(&env, "QuantityUpdated").into_val(&env), // Convert Symbol
        product_id.into_val(&env),                           // Convert u64
    ];
    let expected_data = new_quantity.into_val(&env);

    match event {
        (_, topics, data) if topics == expected_topics && data == expected_data => (),
        _ => panic!(
            "Event does not match expected format. Topics: {:?}, Data: {:?}",
            event.1, event.2
        ),
    }
}

// Auction Tests

#[test]
fn test_create_auction_success() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Apple", "Fruit", "West");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 800u64;
    let auction_end_time = env.ledger().timestamp() + 2 * DAY_SECS; // 2 days

    // create_auction returns Result<(), AuctionError>
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &10,    // min_quantity
        &50,    // bulk_discount_threshold
        &10,    // bulk_discount_percentage (10%)
        &false, // dynamic_pricing
    );
    // No .is_ok() needed

    // Verify using get_auction
    let auction = client.get_auction(&farmer1, &product_id);
    assert_eq!(auction.product_id, product_id);
    assert_eq!(auction.reserve_price, reserve_price);
    assert_eq!(auction.auction_end_time, auction_end_time);
    assert_eq!(auction.highest_bid, 0);
    assert_eq!(auction.highest_bidder, None);
    assert_eq!(auction.farmer, farmer1);
    assert_eq!(auction.quantity_available, details.quantity); // Initial quantity
    assert_eq!(auction.min_quantity, 10);
    assert_eq!(auction.bulk_discount_threshold, 50);
    assert_eq!(auction.bulk_discount_percentage, 10);

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        farmer1.into_val(&env),                             // Convert Address
        Symbol::new(&env, "AuctionCreated").into_val(&env), // Convert Symbol
        product_id.into_val(&env),                          // Convert u64
    ];
    match event {
        (_, topics, _) if topics == expected_topics => (),
        _ => panic!(
            "Event does not match expected format. Topics: {:?}",
            event.1
        ),
    }
}

#[test]
fn test_create_auction_already_exists() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Apple", "Fruit", "West");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 800u64;
    let auction_end_time = env.ledger().timestamp() + 2 * DAY_SECS;

    // Create first auction
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &10,
        &50,
        &10,
        &false,
    );

    // Attempt to create again
    let result = client.try_create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &10,
        &50,
        &10,
        &false,
    );
    match result {
        Err(Ok(e)) if (e) == AuctionError::AuctionAlreadyExists => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_create_auction_product_not_found() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let reserve_price = 800u64;
    let auction_end_time = env.ledger().timestamp() + 2 * DAY_SECS;
    let non_existent_product_id = 999u64;

    let result = client.try_create_auction(
        &farmer1,
        &non_existent_product_id,
        &reserve_price,
        &auction_end_time,
        &10,
        &50,
        &10,
        &false,
    );
    match result {
        Err(Ok(e)) if (e) == AuctionError::ProductNotFound => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_create_auction_invalid_end_time() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Apple", "Fruit", "West");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 800u64;

    // End time in the past
    let past_end_time = env.ledger().timestamp() - DAY_SECS;
    let result_past = client.try_create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &past_end_time,
        &10,
        &50,
        &10,
        &false,
    );
    match result_past {
        Err(Ok(e)) if (e) == AuctionError::InvalidAuctionEndTime => (),
        _ => panic!("Result does not match expected error: {:?}", result_past),
    }

    // End time after product expiry (assuming default expiry is 14 days for Fruit)
    let product = client.get_product(&farmer1, &product_id);
    let late_end_time = product.expiry_date + DAY_SECS;
    let result_late = client.try_create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &late_end_time,
        &10,
        &50,
        &10,
        &false,
    );
    match result_late {
        Err(Ok(e)) if (e) == AuctionError::InvalidAuctionEndTime => (),
        _ => panic!("Result does not match expected error: {:?}", result_late),
    }
}

#[test]
fn test_place_bid_success() {
    let (env, _, client, _, farmer1, _, bidder1, _) = setup_test();
    let details = create_product_details(&env, "Grape", "Fruit", "East");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 1000u64; // Per unit price
    let auction_end_time = env.ledger().timestamp() + 3 * DAY_SECS;
    let min_quantity = 5u32;
    let bid_quantity = 10u32;
    let bid_amount_per_unit = 1100u64; // Higher than reserve
    let total_bid_amount = bid_amount_per_unit * (bid_quantity as u64);

    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &min_quantity,
        &50, // bulk threshold
        &0,  // no discount
        &false,
    );

    // Place bid - returns bool or panics
    let bid_successful = client.place_bid(
        &product_id,
        &total_bid_amount,
        &bid_quantity,
        &bidder1,
        &farmer1,
    );
    assert!(bid_successful); // Check the boolean return value

    // Verify auction state
    let auction = client.get_auction(&farmer1, &product_id);
    assert_eq!(auction.highest_bid, total_bid_amount);
    assert_eq!(auction.highest_bidder, Some(bidder1.clone()));

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        farmer1.into_val(&env),                     // Convert Address
        Symbol::new(&env, "NewBid").into_val(&env), // Convert Symbol
        product_id.into_val(&env),                  // Convert u64
    ];
    let expected_data = (bidder1, total_bid_amount, bid_quantity).into_val(&env);
    match event {
        (_, topics, data) if topics == expected_topics && data == expected_data => (),
        _ => panic!(
            "Event does not match expected format. Topics: {:?}, Data: {:?}",
            event.1, event.2
        ),
    }
}

#[test]
fn test_place_bid_with_bulk_discount() {
    let (env, _, client, _, farmer1, _, bidder1, _) = setup_test();
    let details = create_product_details(&env, "Potato", "Root_Vegetables", "Central");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 500u64; // Per unit
    let auction_end_time = env.ledger().timestamp() + DAY_SECS;
    let min_quantity = 10u32;
    let bulk_threshold = 20u32;
    let bulk_discount = 10u32; // 10%

    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &min_quantity,
        &bulk_threshold,
        &bulk_discount,
        &false,
    );

    // Bid qualifies for discount
    let bid_quantity = 25u32;
    let bid_amount_per_unit = 600u64;
    let total_bid_amount = bid_amount_per_unit * (bid_quantity as u64); // 15000
    let expected_discount = total_bid_amount * (bulk_discount as u64) / 100; // 1500
    let expected_effective_bid = total_bid_amount - expected_discount; // 13500

    let bid_successful = client.place_bid(
        &product_id,
        &total_bid_amount,
        &bid_quantity,
        &bidder1,
        &farmer1,
    );
    assert!(bid_successful);

    let auction = client.get_auction(&farmer1, &product_id);
    assert_eq!(auction.highest_bid, expected_effective_bid); // Stores discounted bid
    assert_eq!(auction.highest_bidder, Some(bidder1));
}

#[test]
fn test_place_bid_too_low_reserve() {
    let (env, _, client, _, farmer1, _, bidder1, _) = setup_test();
    let details = create_product_details(&env, "Grape", "Fruit", "East");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 1000u64;
    let auction_end_time = env.ledger().timestamp() + 3 * DAY_SECS;
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &5,
        &50,
        &0,
        &false,
    );

    // Bid below reserve
    let bid_quantity = 10u32;
    let bid_amount_per_unit = 900u64; // Below reserve
    let total_bid_amount = bid_amount_per_unit * (bid_quantity as u64);

    let result = client.try_place_bid(
        &product_id,
        &total_bid_amount,
        &bid_quantity,
        &bidder1,
        &farmer1,
    );
    match result {
        Err(Ok(e)) if (e) == AuctionError::BidTooLow => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_place_bid_too_low_highest() {
    let (env, _, client, _, farmer1, _, bidder1, bidder2) = setup_test();
    let details = create_product_details(&env, "Grape", "Fruit", "East");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 1000u64;
    let auction_end_time = env.ledger().timestamp() + 3 * DAY_SECS;
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &5,
        &50,
        &0,
        &false,
    );

    // First bid (successful)
    let bid1_quantity = 10u32;
    let bid1_amount_per_unit = 1100u64;
    let total_bid1_amount = bid1_amount_per_unit * (bid1_quantity as u64);
    client.place_bid(
        &product_id,
        &total_bid1_amount,
        &bid1_quantity,
        &bidder1,
        &farmer1,
    );

    // Second bid (lower than first)
    let bid2_quantity = 10u32;
    let bid2_amount_per_unit = 1050u64; // Higher than reserve, lower than bid1
    let total_bid2_amount = bid2_amount_per_unit * (bid2_quantity as u64);

    let result = client.try_place_bid(
        &product_id,
        &total_bid2_amount,
        &bid2_quantity,
        &bidder2,
        &farmer1,
    );
    match result {
        Err(Ok(e)) if (e) == AuctionError::BidTooLow => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_place_bid_auction_ended() {
    let (env, _, client, _, farmer1, _, bidder1, _) = setup_test();
    let details = create_product_details(&env, "Grape", "Fruit", "East");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 1000u64;
    let auction_end_time = env.ledger().timestamp() + 100; // Ends soon
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &5,
        &50,
        &0,
        &false,
    );

    // Advance time past auction end
    advance_time(&env, 200);

    let bid_quantity = 10u32;
    let bid_amount_per_unit = 1100u64;
    let total_bid_amount = bid_amount_per_unit * (bid_quantity as u64);

    let result = client.try_place_bid(
        &product_id,
        &total_bid_amount,
        &bid_quantity,
        &bidder1,
        &farmer1,
    );
    match result {
        Err(Ok(e)) if (e) == AuctionError::AuctionEnded => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_place_bid_farmer_cannot_bid() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Grape", "Fruit", "East");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 1000u64;
    let auction_end_time = env.ledger().timestamp() + DAY_SECS;
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &5,
        &50,
        &0,
        &false,
    );

    let bid_quantity = 10u32;
    let bid_amount_per_unit = 1100u64;
    let total_bid_amount = bid_amount_per_unit * (bid_quantity as u64);

    // Farmer tries to bid
    let result = client.try_place_bid(
        &product_id,
        &total_bid_amount,
        &bid_quantity,
        &farmer1, // Bidder is farmer
        &farmer1,
    );
    match result {
        Err(Ok(e)) if (e) == AuctionError::InvalidBidder => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_extend_auction_success() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Orange", "Citrus", "South");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 700u64;
    let auction_end_time = env.ledger().timestamp() + 2 * DAY_SECS;
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &10,
        &50,
        &0,
        &false,
    );

    let new_end_time = auction_end_time + DAY_SECS; // Extend by 1 day
                                                    // extend_auction returns Result<(), AuctionError>
    client.extend_auction(&farmer1, &product_id, &new_end_time);
    // No .is_ok() needed

    let auction = client.get_auction(&farmer1, &product_id);
    assert_eq!(auction.auction_end_time, new_end_time);

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        farmer1.into_val(&env),                              // Convert Address
        Symbol::new(&env, "AuctionExtended").into_val(&env), // Convert Symbol
        product_id.into_val(&env),                           // Convert u64
    ];
    let expected_data = new_end_time.into_val(&env);
    match event {
        (_, topics, data) if topics == expected_topics && data == expected_data => (),
        _ => panic!(
            "Event does not match expected format. Topics: {:?}, Data: {:?}",
            event.1, event.2
        ),
    }
}

#[test]
fn test_extend_auction_already_ended() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Orange", "Citrus", "South");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 700u64;
    let auction_end_time = env.ledger().timestamp() + 100;
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &10,
        &50,
        &0,
        &false,
    );

    advance_time(&env, 200); // End auction

    let new_end_time = auction_end_time + DAY_SECS;
    let result = client.try_extend_auction(&farmer1, &product_id, &new_end_time);
    match result {
        Err(Ok(e)) if (e) == AuctionError::AuctionEnded => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_finalize_auction_success() {
    let (env, contract_id, client, _, farmer1, _, bidder1, _) = setup_test(); // Need contract_id for event check
    let details = create_product_details(&env, "Lemon", "Citrus", "West");
    let initial_quantity = details.quantity;
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 600u64;
    let auction_end_time = env.ledger().timestamp() + 100;
    let min_quantity = 20u32;
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &min_quantity,
        &50,
        &0,
        &false,
    );

    // Place a winning bid
    let bid_quantity = min_quantity;
    let total_bid_amount = 700 * (bid_quantity as u64);
    client.place_bid(
        &product_id,
        &total_bid_amount,
        &bid_quantity,
        &bidder1,
        &farmer1,
    );

    advance_time(&env, 200); // End auction

    // Finalize - returns Result<(), AuctionError>
    client.finalize_auction(&farmer1, &product_id);
    // No .is_ok() needed

    // Verify auction is removed
    let get_auction_result = client.try_get_auction(&farmer1, &product_id);
    match get_auction_result {
        Err(Ok(e)) if (e) == AuctionError::AuctionNotFound => (),
        _ => panic!(
            "Auction was not removed or other error occurred: {:?}",
            get_auction_result
        ),
    }

    // Verify product quantity is reduced
    let product = client.get_product(&farmer1, &product_id);
    // In the contract logic, it seems auction.quantity_available is used, which is set to product.quantity at auction creation
    let auction_quantity = initial_quantity; // Should be the quantity available at auction creation
    assert_eq!(
        product.quantity,
        initial_quantity.saturating_sub(auction_quantity) // Should be 0 if full quantity auctioned
    );

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        farmer1.into_val(&env), // Convert Address
        Symbol::new(&env, "AuctionFinalized").into_val(&env), // Convert Symbol
        product_id.into_val(&env), // Convert u64
    ];
    // Event data is a tuple (Option<Address>, u64) -> (Address, u64) because we know there's a winner
    let expected_data = (bidder1, total_bid_amount).into_val(&env);
    match event {
        (contract_id_val, topics, data)
            if contract_id_val == contract_id.into_val(&env)
                && topics == expected_topics
                && data == expected_data.into_val(&env) =>
        {
            ()
        }
        _ => panic!(
            "Event does not match expected format. Contract: {:?}, Topics: {:?}, Data: {:?}",
            event.0, event.1, event.2
        ),
    }
}

#[test]
fn test_finalize_auction_not_yet_ended() {
    let (env, _, client, _, farmer1, _, bidder1, _) = setup_test();
    let details = create_product_details(&env, "Lemon", "Citrus", "West");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 600u64;
    let auction_end_time = env.ledger().timestamp() + DAY_SECS; // Ends in future
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &20,
        &50,
        &0,
        &false,
    );
    client.place_bid(&product_id, &(700 * 20), &20, &bidder1, &farmer1);

    let result = client.try_finalize_auction(&farmer1, &product_id);
    match result {
        Err(Ok(e)) if (e) == AuctionError::AuctionNotYetEnded => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_finalize_auction_no_bids() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let details = create_product_details(&env, "Lemon", "Citrus", "West");
    let product_id = client.add_product(&farmer1, &details);
    let reserve_price = 600u64;
    let auction_end_time = env.ledger().timestamp() + 100;
    client.create_auction(
        &farmer1,
        &product_id,
        &reserve_price,
        &auction_end_time,
        &20,
        &50,
        &0,
        &false,
    );

    advance_time(&env, 200); // End auction

    let result = client.try_finalize_auction(&farmer1, &product_id);
    match result {
        Err(Ok(e)) if (e) == AuctionError::NoBidsPlaced => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

// Price Oracle Tests

#[test]
fn test_update_and_fetch_market_price() {
    let (env, contract_id, client, admin, _, _, _, _) = setup_test(); // Need contract_id
    let product_type = Symbol::new(&env, "Corn");
    let region = Symbol::new(&env, "Midwest");
    let price = 150u64;
    let trend = 1i32; // Rising
    let volume = 10000u64;

    // Update price - returns Result<(), OracleError>
    client.update_market_price(&admin, &product_type, &region, &price, &trend, &volume);
    // No .is_ok() needed

    // Fetch price
    let market_price = client.fetch_market_price(&product_type, &region);
    assert_eq!(market_price.product_type, product_type);
    assert_eq!(market_price.region, region);
    assert_eq!(market_price.price, price);
    assert_eq!(market_price.trend, trend);
    assert_eq!(market_price.volume, volume);
    assert!(market_price.timestamp > 0);

    // Check event
    let event = env.events().all().last().unwrap();
    let expected_topics = vec![
        &env,
        Symbol::new(&env, "MarketPriceUpdated").into_val(&env), // Convert Symbol
        product_type.into_val(&env),                            // Convert Symbol
        region.into_val(&env),                                  // Convert Symbol
    ];
    // Event data is just the price (u64)
    let expected_data = price.into_val(&env);
    match event {
        (contract_id_val, topics, data)
            if contract_id_val == contract_id.into_val(&env)
                && topics == expected_topics
                && data == expected_data.into_val(&env) =>
        {
            ()
        }
        _ => panic!(
            "Event does not match expected format. Contract: {:?}, Topics: {:?}, Data: {:?}",
            event.0, event.1, event.2
        ),
    }
}

#[test]
fn test_update_market_price_unauthorized() {
    let (env, contract_id, client, _, farmer1, _, _, _) = setup_test(); // Use farmer1 (not admin), need contract_id
    let product_type = Symbol::new(&env, "Corn");
    let region = Symbol::new(&env, "Midwest");
    let price = 150u64;

    // farmer1 tries to update
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &farmer1, // Only authorize farmer1
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id, // Use the actual contract Address (ID)
            fn_name: "update_market_price",
            args: vec![
                &env,
                farmer1.into_val(&env),
                product_type.into_val(&env),
                region.into_val(&env),
                price.into_val(&env),
                1i32.into_val(&env),
                1000u64.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let result = client.try_update_market_price(&farmer1, &product_type, &region, &price, &1, &1000);
    // It should fail because farmer1 != stored admin
    match result {
        Err(Ok(e)) if (e) == OracleError::InvalidPriceData => (),
        // It could also fail authorization at a higher level depending on mock_auths setup vs require_auth details
        Err(Err(_host_error)) => { /* Could be a host error like Auth */ }
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_fetch_market_price_not_available() {
    let (env, _, client, _, _, _, _, _) = setup_test();
    let product_type = Symbol::new(&env, "Wheat");
    let region = Symbol::new(&env, "Plains");

    let result = client.try_fetch_market_price(&product_type, &region);
    match result {
        Err(Ok(e)) if (e) == OracleError::PriceDataNotAvailable => (),
        _ => panic!("Result does not match expected error: {:?}", result),
    }
}

#[test]
fn test_compare_with_market() {
    let (env, _, client, admin, farmer1, _, _, _) = setup_test();
    let product_type = Symbol::new(&env, "Soybeans");
    let region = Symbol::new(&env, "South");
    let market_price_val = 200u64;

    // Set market price
    client.update_market_price(&admin, &product_type, &region, &market_price_val, &0, &5000);

    // Add product with different price
    let mut details = create_product_details(&env, "Soybeans", "Grains", "South");
    details.base_price = 220; // 10% higher than market
    details.product_type = product_type.clone();
    details.region = region.clone();
    let product_id = client.add_product(&farmer1, &details);

    // Compare - returns i32 or panics
    let difference = client.compare_with_market(&farmer1, &product_id);
    // Expected: ((220 - 200) * 100) / 200 = 2000 / 200 = 10
    assert_eq!(difference, 10);

    // Update product price to be lower - need to add a new product or update existing
    // Let's add a new one for simplicity in this test setup
    details.base_price = 180; // 10% lower
    let product_id_2 = client.add_product(&farmer1, &details);
    let difference_2 = client.compare_with_market(&farmer1, &product_id_2);
    // Expected: ((180 - 200) * 100) / 200 = -2000 / 200 = -10
    assert_eq!(difference_2, -10);
}

#[test]
fn test_suggest_price() {
    let (env, _, client, admin, _, _, _, _) = setup_test();
    let product_type = Symbol::new(&env, "Berries");
    let region = Symbol::new(&env, "Northwest");
    let market_price_val = 500u64;

    // Set market price
    client.update_market_price(&admin, &product_type, &region, &market_price_val, &0, &1000);
    // Assume InSeason for now (default if not set in verify_seasonal_status)

    // Suggest for Premium Quality, Excellent Freshness
    let quality = Symbol::new(&env, "Premium"); // +30% Quality
    let freshness = Symbol::new(&env, "Excellent"); // +10% Freshness
                                                    // Calculation based on contract logic (integer math):
                                                    // Base = 500
                                                    // Quality Adj = 500 + (500 * 30 / 100) = 650
                                                    // Freshness Adj = 650 + (650 * 10 / 100) = 715
                                                    // Seasonal Adj (assuming YearRound/InSeason - default/small discount) = 715 - (715 * 5 / 100) = 680
                                                    // Trend Adj (trend=0) = 680

    let suggested_price = client.suggest_price(&product_type, &region, &quality, &freshness);
    assert_eq!(suggested_price, 680);

    // Suggest for Grade C, Fair Freshness
    let quality_c = Symbol::new(&env, "Grade_C"); // -15% Quality
    let freshness_f = Symbol::new(&env, "Fair"); // -10% Freshness
                                                 // Calculation based on contract logic:
                                                 // Base = 500
                                                 // Quality Adj = 500 - (500 * 15 / 100) = 425
                                                 // Freshness Adj = 425 - (425 * 10 / 100) = 383
                                                 // Seasonal Adj = 383 - (383 * 5 / 100) = 364
                                                 // Trend Adj = 364
    let suggested_price_c = client.suggest_price(&product_type, &region, &quality_c, &freshness_f);
    assert_eq!(suggested_price_c, 364);
}

// Time Management Tests

#[test]
fn test_update_product_freshness_and_price() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    let mut details = create_product_details(&env, "Lettuce", "Leafy_Greens", "Valley");
    details.harvest_date = env.ledger().timestamp() - 4 * DAY_SECS; // Harvested 4 days ago
    let initial_base_price = details.base_price; // Store base price for comparison
    let product_id = client.add_product(&farmer1, &details);

    // Initial state check (should be Excellent)
    let initial_product = client.get_product(&farmer1, &product_id);
    assert_eq!(initial_product.freshness_rating, FreshnessRating::Excellent);
    // Price adjust based on base: Base + 10% = 1000 + 100 = 1100
    assert_eq!(initial_product.current_price, 1100);

    // Advance time to make it "Good" (e.g., 7 days old)
    advance_time(&env, 3 * DAY_SECS); // Total 7 days old

    // Call update - returns FreshnessRating or panics
    let new_rating = client.update_product_freshness(&farmer1, &product_id);
    assert_eq!(new_rating, FreshnessRating::Good);

    // Verify stored state
    let updated_product = client.get_product(&farmer1, &product_id);
    assert_eq!(updated_product.freshness_rating, FreshnessRating::Good);
    // Price adjust based on base: Base (Good has no adjustment from base) = 1000
    assert_eq!(updated_product.current_price, initial_base_price); // Back to base price
}

#[test]
fn test_check_product_expiry() {
    let (env, _, client, _, farmer1, _, _, _) = setup_test();
    // Leafy greens expire in 7 days
    let details = create_product_details(&env, "Spinach", "Leafy_Greens", "Coast");
    let product_id = client.add_product(&farmer1, &details);

    // Check before expiry
    advance_time(&env, 6 * DAY_SECS);
    let is_expired1 = client.check_product_expiry(&farmer1, &product_id);
    assert_eq!(is_expired1, false);

    // Check after expiry
    advance_time(&env, 2 * DAY_SECS); // Total 8 days old
    let is_expired2 = client.check_product_expiry(&farmer1, &product_id);
    assert_eq!(is_expired2, true);
}
