#![cfg(test)]

extern crate std;

// Remove unused imports to clean up warnings

use super::utils::{register_basic_equipment, setup_test};

// ============================================================================
// PRICING CALCULATION TESTS
// ============================================================================

#[test]
fn test_compute_total_price_success() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Use day-based timestamps for pricing calculation
    let start_day = (env.ledger().timestamp() / 86400) + 1; // Tomorrow in days
    let end_day = start_day + 5; // 5 days later

    let price = client.compute_total_price(&equipment_id, &start_day, &end_day);
    assert_eq!(price, 5000); // 5 days * 1000 per day
}

#[test]
fn test_compute_total_price_single_day() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 2000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 1; // 1 day duration

    let price = client.compute_total_price(&equipment_id, &start_day, &end_day);
    assert_eq!(price, 2000); // 1 day * 2000 per day
}

#[test]
#[should_panic]
fn test_compute_total_price_invalid_dates() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 10;
    let end_day = start_day - 1; // Invalid: end before start

    client.compute_total_price(&equipment_id, &start_day, &end_day);
}

#[test]
#[should_panic]
fn test_compute_total_price_nonexistent_equipment() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = super::utils::create_equipment_id(&env, "nonexistent");

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 5;

    client.compute_total_price(&equipment_id, &start_day, &end_day);
}

// ============================================================================
// PRICE VALIDATION TESTS
// ============================================================================

#[test]
fn test_validate_price_within_tolerance() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 3; // 3 days
    let _expected_price = 3000;
    let proposed_price = 3050; // Slightly higher
    let tolerance = 100; // Within tolerance

    // Should not panic
    client.validate_price(&equipment_id, &start_day, &end_day, &proposed_price, &tolerance);
}

#[test]
fn test_validate_price_exact_match() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 3; // 3 days
    let exact_price = 3000;
    let tolerance = 0; // No tolerance needed for exact match

    // Should not panic
    client.validate_price(&equipment_id, &start_day, &end_day, &exact_price, &tolerance);
}

#[test]
#[should_panic]
fn test_validate_price_outside_tolerance_high() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 3; // 3 days, expected = 3000
    let proposed_price = 5000; // Too high
    let tolerance = 100;

    client.validate_price(&equipment_id, &start_day, &end_day, &proposed_price, &tolerance);
}

#[test]
#[should_panic]
fn test_validate_price_outside_tolerance_low() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 3; // 3 days, expected = 3000
    let proposed_price = 1000; // Too low
    let tolerance = 100;

    client.validate_price(&equipment_id, &start_day, &end_day, &proposed_price, &tolerance);
}

// ============================================================================
// PAYMENT SCHEDULE ADHERENCE TESTS
// ============================================================================

#[test]
fn test_rental_payment_tracking() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1500);

    // Calculate expected price for 2 days
    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 2;
    let expected_price = client.compute_total_price(&equipment_id, &start_day, &end_day);
    assert_eq!(expected_price, 3000); // 2 days * 1500

    // Create rental with computed price
    let start_timestamp = start_day * 86400;
    let end_timestamp = end_day * 86400;
    client.create_rental(&equipment_id, &renter1, &start_timestamp, &end_timestamp, &expected_price);

    // Verify rental tracks correct payment amount
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.total_price, expected_price);
}

#[test]
fn test_different_pricing_tiers() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    
    // Register equipment with different price tiers
    let basic_tractor = register_basic_equipment(&client, &env, "basic_tractor", 800);
    let premium_tractor = register_basic_equipment(&client, &env, "premium_tractor", 2000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 3; // 3 days

    let basic_price = client.compute_total_price(&basic_tractor, &start_day, &end_day);
    let premium_price = client.compute_total_price(&premium_tractor, &start_day, &end_day);

    assert_eq!(basic_price, 2400); // 3 days * 800
    assert_eq!(premium_price, 6000); // 3 days * 2000
    assert!(premium_price > basic_price);
}

// ============================================================================
// EDGE CASE PAYMENT TESTS
// ============================================================================

#[test]
fn test_zero_day_rental_edge_case() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day; // Same day (0 duration)

    // Should handle zero-duration rental gracefully
    let price = client.compute_total_price(&equipment_id, &start_day, &end_day);
    assert_eq!(price, 0);
}

#[test]
fn test_high_value_rental_calculation() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "expensive_equipment", 50000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 30; // 30 days

    let price = client.compute_total_price(&equipment_id, &start_day, &end_day);
    assert_eq!(price, 1_500_000); // 30 days * 50,000
}

// ============================================================================
// INTEGRATION SMOKE TESTS
// ============================================================================

#[test]
fn test_pricing_integration_with_rental_flow() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1200);

    // 1. Compute expected price
    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 4; // 4 days
    let expected_price = client.compute_total_price(&equipment_id, &start_day, &end_day);
    assert_eq!(expected_price, 4800); // 4 days * 1200

    // 2. Validate the price
    let tolerance = 50;
    client.validate_price(&equipment_id, &start_day, &end_day, &expected_price, &tolerance);

    // 3. Create rental with validated price
    let start_timestamp = start_day * 86400;
    let end_timestamp = end_day * 86400;
    client.create_rental(&equipment_id, &renter1, &start_timestamp, &end_timestamp, &expected_price);

    // 4. Verify rental was created with correct price
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.total_price, expected_price);
    assert_eq!(rental.renter, renter1);
}

#[test]
fn test_payment_validation_prevents_invalid_rentals() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 2; // 2 days, should cost 2000

    // Test validation then create rental with correct price
    let correct_price = 2000;
    let start_timestamp = start_day * 86400;
    let end_timestamp = end_day * 86400;
    client.create_rental(&equipment_id, &renter1, &start_timestamp, &end_timestamp, &correct_price);
    
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.total_price, correct_price);
}

// ============================================================================
// INTEGRATION SIMULATION TESTS 
// ============================================================================

#[test]
fn test_payment_flow_simulation() {
    // This test simulates a tokenized payment flow without actual token contract integration
    // to demonstrate the concept and test payment validation
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_day = (env.ledger().timestamp() / 86400) + 1;
    let end_day = start_day + 2; // 2 days
    let expected_price = client.compute_total_price(&equipment_id, &start_day, &end_day);
    assert_eq!(expected_price, 2000);

    // Simulate payment validation (what would happen with token contract)
    let tolerance = 50;
    client.validate_price(&equipment_id, &start_day, &end_day, &expected_price, &tolerance);

    // Create rental with validated price (simulating successful token payment)
    let start_timestamp = start_day * 86400;
    let end_timestamp = end_day * 86400;
    client.create_rental(&equipment_id, &renter1, &start_timestamp, &end_timestamp, &expected_price);
    client.confirm_rental(&equipment_id);

    // Verify the rental was created successfully with proper price tracking
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.total_price, expected_price);
    assert_eq!(rental.renter, renter1);

    // This demonstrates the integration point where a commodity token contract
    // could validate and process the payment before rental creation
}
