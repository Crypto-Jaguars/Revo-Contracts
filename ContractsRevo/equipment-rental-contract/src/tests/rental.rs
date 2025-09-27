#![cfg(test)]

use crate::{equipment::MaintenanceStatus, rental::RentalStatus};

use super::utils::{register_basic_equipment, setup_test, create_standard_rental};

// ============================================================================
// RENTAL AGREEMENT CREATION TESTS
// ============================================================================

#[test]
fn test_create_rental_success() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    client.create_rental(&equipment_id, &renter1, &start_date, &end_date, &total_price);

    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.equipment_id, equipment_id);
    assert_eq!(rental.renter, renter1);
    assert_eq!(rental.start_date, start_date);
    assert_eq!(rental.end_date, end_date);
    assert_eq!(rental.total_price, total_price);
    assert_eq!(rental.status, RentalStatus::Pending);
}

#[test]
#[should_panic(expected = "Equipment not available")]
fn test_create_rental_unavailable_equipment() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Make equipment unavailable
    client.update_availability(&equipment_id, &false);

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    client.create_rental(&equipment_id, &renter1, &start_date, &end_date, &total_price);
}

#[test]
#[should_panic(expected = "Equipment under maintenance or needs service")]
fn test_create_rental_equipment_under_maintenance() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Set equipment under maintenance
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::UnderMaintenance);

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    client.create_rental(&equipment_id, &renter1, &start_date, &end_date, &total_price);
}

#[test]
#[should_panic(expected = "Rental already exists for this equipment")]
fn test_create_rental_double_booking() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    client.create_rental(&equipment_id, &renter1, &start_date, &end_date, &total_price);
    // Attempt double booking
    client.create_rental(&equipment_id, &renter1, &start_date, &end_date, &total_price);
}

// ============================================================================
// RENTAL LIFECYCLE MANAGEMENT TESTS
// ============================================================================

#[test]
fn test_confirm_rental_success() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);

    client.confirm_rental(&equipment_id);

    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Active);
}

#[test]
#[should_panic(expected = "Rental not pending")]
fn test_confirm_rental_not_pending() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.confirm_rental(&equipment_id);

    // Try to confirm again (should fail)
    client.confirm_rental(&equipment_id);
}

#[test]
fn test_complete_rental_success() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.confirm_rental(&equipment_id);
    client.complete_rental(&equipment_id);

    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Completed);

    // Equipment should be available again
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert!(equipment.available);
}

#[test]
#[should_panic(expected = "Rental not active")]
fn test_complete_rental_not_active() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    
    // Try to complete without confirming first
    client.complete_rental(&equipment_id);
}

#[test]
fn test_cancel_rental_success() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.cancel_rental(&equipment_id);

    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Cancelled);

    // Verify history is updated
    let history = client.get_rental_history_by_equipment(&equipment_id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().status, RentalStatus::Cancelled);

    // Should be able to create new rental after cancellation
    let new_start_date = env.ledger().timestamp() + (10 * 86400);
    let new_end_date = new_start_date + (2 * 86400);
    let new_total_price = 2000;
    
    client.create_rental(&equipment_id, &renter1, &new_start_date, &new_end_date, &new_total_price);
    
    let new_rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(new_rental.status, RentalStatus::Pending);
}

#[test]
#[should_panic(expected = "Only pending rentals can be cancelled")]
fn test_cancel_rental_already_active() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.confirm_rental(&equipment_id);

    // Try to cancel active rental
    client.cancel_rental(&equipment_id);
}

// ============================================================================
// RENTAL HISTORY TESTS
// ============================================================================

#[test]
fn test_rental_history_by_equipment() {
    let (env, _contract_id, client, _owner, renter1, renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // First rental - complete it
    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.confirm_rental(&equipment_id);
    client.complete_rental(&equipment_id);

    // Second rental - leave pending
    let start_date2 = env.ledger().timestamp() + (10 * 86400);
    let end_date2 = start_date2 + (2 * 86400);
    let total_price2 = 2000;
    client.create_rental(&equipment_id, &renter2, &start_date2, &end_date2, &total_price2);

    let history = client.get_rental_history_by_equipment(&equipment_id);
    assert_eq!(history.len(), 2);
    
    let first_rental = history.get(0).unwrap();
    let second_rental = history.get(1).unwrap();
    
    assert_eq!(first_rental.renter, renter1);
    assert_eq!(first_rental.status, RentalStatus::Completed);
    assert_eq!(second_rental.renter, renter2);
    assert_eq!(second_rental.status, RentalStatus::Pending);
}

#[test]
fn test_rental_history_by_user() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id1 = register_basic_equipment(&client, &env, "tractor_001", 1000);
    let equipment_id2 = register_basic_equipment(&client, &env, "harvester_001", 1500);

    // Create rentals for same user on different equipment
    create_standard_rental(&client, &env, &equipment_id1, &renter1, 3);
    create_standard_rental(&client, &env, &equipment_id2, &renter1, 2);

    let user_history = client.get_rental_history_by_user(&renter1);
    assert_eq!(user_history.len(), 2);

    // Both rentals should belong to same user
    assert_eq!(user_history.get(0).unwrap().renter, renter1);
    assert_eq!(user_history.get(1).unwrap().renter, renter1);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_rental_lifecycle() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // 1. Verify initial state
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true);
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::Good);

    // 2. Create rental
    let (_start_date, _end_date, total_price) = create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Pending);
    assert_eq!(rental.total_price, total_price);

    // 3. Confirm rental
    client.confirm_rental(&equipment_id);
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Active);

    // 4. Complete rental
    client.complete_rental(&equipment_id);
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Completed);

    // 5. Verify equipment is available again
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true);

    // 6. Verify rental history
    let history = client.get_rental_history_by_equipment(&equipment_id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().status, RentalStatus::Completed);
}