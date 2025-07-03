#![cfg(test)]

extern crate std;

use super::*;
use crate::equipment::MaintenanceStatus;
use crate::rental::RentalStatus;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, BytesN, Env, String,
};

/// Test setup helper function
/// Returns: (env, contract_id, client, owner, renter1, renter2)
fn setup_test<'a>() -> (
    Env,
    Address,
    EquipmentRentalContractClient<'a>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let renter1 = Address::generate(&env);
    let renter2 = Address::generate(&env);

    let contract_id = env.register(EquipmentRentalContract, ());
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    (env, contract_id, client, owner, renter1, renter2)
}

/// Helper function to create equipment ID
fn create_equipment_id(env: &Env, id: &str) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    let id_bytes = id.as_bytes();
    bytes[..id_bytes.len().min(32)].copy_from_slice(&id_bytes[..id_bytes.len().min(32)]);
    BytesN::from_array(env, &bytes)
}

/// Helper function to advance ledger time
fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|li| {
        li.timestamp += seconds;
    });
}

// ============================================================================
// EQUIPMENT REGISTRATION TESTS
// ============================================================================

#[test]
fn test_register_equipment_success() {
    let (env, contract_id, client, owner, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    // Verify equipment was registered correctly
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.id, equipment_id);
    assert_eq!(equipment.equipment_type, equipment_type);
    assert_eq!(equipment.owner, contract_id); // Contract address is the owner
    assert_eq!(equipment.rental_price_per_day, rental_price_per_day);
    assert_eq!(equipment.location, location);
    assert_eq!(equipment.available, true); // Default availability
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::Good); // Default status
}

#[test]
#[should_panic(expected = "Equipment already registered")]
fn test_register_equipment_duplicate() {
    let (env, _, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment first time
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    // Try to register same equipment again - should panic
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );
}

#[test]
fn test_get_equipment_nonexistent() {
    let (env, _, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "nonexistent");

    // Try to get non-existent equipment
    let equipment = client.get_equipment(&equipment_id);
    assert!(equipment.is_none());
}

#[test]
fn test_update_equipment_availability() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    // Update availability to false
    client.update_availability(&equipment_id, &false);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, false);

    // Update availability back to true
    client.update_availability(&equipment_id, &true);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true);
}

#[test]
fn test_update_maintenance_status() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    // Update to NeedsService
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::NeedsService);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(
        equipment.maintenance_status,
        MaintenanceStatus::NeedsService
    );

    // Update to UnderMaintenance
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::UnderMaintenance);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(
        equipment.maintenance_status,
        MaintenanceStatus::UnderMaintenance
    );

    // Update back to Good
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::Good);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::Good);
}

// ============================================================================
// RENTAL FLOW TESTS
// ============================================================================

#[test]
fn test_create_rental_success() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = env.ledger().timestamp() + 86400; // Tomorrow
    let end_date = start_date + (3 * 86400); // 3 days later
    let total_price = 3000; // 3 days * 1000

    // Create rental
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );

    // Verify rental was created
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.equipment_id, equipment_id);
    assert_eq!(rental.renter, renter1);
    assert_eq!(rental.start_date, start_date);
    assert_eq!(rental.end_date, end_date);
    assert_eq!(rental.total_price, total_price);
    assert_eq!(rental.status, RentalStatus::Pending);

    // Note: Equipment remains available after creating rental (contract design)
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true);
}

#[test]
#[should_panic(expected = "Equipment not available")]
fn test_create_rental_unavailable_equipment() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    // Make equipment unavailable
    client.update_availability(&equipment_id, &false);

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Try to create rental - should panic
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );
}

#[test]
#[should_panic(expected = "Equipment under maintenance or needs service")]
fn test_create_rental_equipment_under_maintenance() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    // Set equipment under maintenance
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::UnderMaintenance);

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Try to create rental - should panic
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );
}

#[test]
#[should_panic(expected = "Rental already exists for this equipment")]
fn test_create_rental_already_exists() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Create first rental
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );

    // Try to create second rental for same equipment - should panic
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );
}

#[test]
fn test_confirm_rental_success() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Create rental
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );

    // Confirm rental
    client.confirm_rental(&equipment_id);

    // Verify rental status changed to Active
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Active);
}

#[test]
fn test_complete_rental_success() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Create and confirm rental
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );
    client.confirm_rental(&equipment_id);

    // Complete rental
    client.complete_rental(&equipment_id);

    // Verify rental status changed to Completed
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Completed);

    // Verify equipment is available again
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true);
}

#[test]
fn test_cancel_rental_success() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Create rental
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );

    // Cancel rental
    client.cancel_rental(&equipment_id);

    // Verify rental status changed to Cancelled
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Cancelled);
}

#[test]
#[should_panic(expected = "Only pending rentals can be cancelled")]
fn test_cancel_rental_already_active() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Create and confirm rental
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );
    client.confirm_rental(&equipment_id);

    // Try to cancel active rental - should panic
    client.cancel_rental(&equipment_id);
}

#[test]
fn test_rental_history_by_equipment() {
    let (env, contract_id, client, _, renter1, renter2) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date1 = env.ledger().timestamp() + 86400;
    let end_date1 = start_date1 + (3 * 86400);
    let total_price1 = 3000;

    // Create first rental
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date1,
        &end_date1,
        &total_price1,
    );
    client.confirm_rental(&equipment_id);
    client.complete_rental(&equipment_id);

    // Create second rental for the same equipment (after completion)
    let start_date2 = env.ledger().timestamp() + (10 * 86400);
    let end_date2 = start_date2 + (2 * 86400);
    let total_price2 = 2000;

    client.create_rental(
        &equipment_id,
        &renter2,
        &start_date2,
        &end_date2,
        &total_price2,
    );

    // Get rental history for equipment
    let history = client.get_rental_history_by_equipment(&equipment_id);
    assert_eq!(history.len(), 2);

    // Verify first rental
    let first_rental = history.get(0).unwrap();
    assert_eq!(first_rental.renter, renter1);
    assert_eq!(first_rental.status, RentalStatus::Completed);

    // Verify second rental in the same history
    let second_rental = history.get(1).unwrap();
    assert_eq!(second_rental.renter, renter2);
    assert_eq!(second_rental.status, RentalStatus::Pending);
}

#[test]
fn test_rental_history_by_user() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id1 = create_equipment_id(&env, "tractor_001");
    let equipment_id2 = create_equipment_id(&env, "harvester_001");
    let equipment_type = String::from_str(&env, "Agricultural Equipment");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register two pieces of equipment
    client.register_equipment(
        &equipment_id1,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );
    client.register_equipment(
        &equipment_id2,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date1 = env.ledger().timestamp() + 86400;
    let end_date1 = start_date1 + (3 * 86400);
    let total_price1 = 3000;

    let start_date2 = env.ledger().timestamp() + (10 * 86400);
    let end_date2 = start_date2 + (2 * 86400);
    let total_price2 = 2000;

    // Create rentals for same user on different equipment
    client.create_rental(
        &equipment_id1,
        &renter1,
        &start_date1,
        &end_date1,
        &total_price1,
    );
    client.create_rental(
        &equipment_id2,
        &renter1,
        &start_date2,
        &end_date2,
        &total_price2,
    );

    // Get rental history for user
    let history = client.get_rental_history_by_user(&renter1);
    assert_eq!(history.len(), 2);

    // Verify both rentals belong to the same user
    assert_eq!(history.get(0).unwrap().renter, renter1);
    assert_eq!(history.get(1).unwrap().renter, renter1);
}

// ============================================================================
// PRICING TESTS
// ============================================================================

#[test]
fn test_compute_total_price() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = (env.ledger().timestamp() / 86400) + 1; // Tomorrow in days
    let end_date = start_date + 5; // 5 days later

    // Compute total price - this should return i128 directly
    let price = client.compute_total_price(&equipment_id, &start_date, &end_date);
    assert_eq!(price, 5000); // 5 days * 1000
}

#[test]
fn test_validate_price_success() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = (env.ledger().timestamp() / 86400) + 1; // Tomorrow in days
    let end_date = start_date + 3; // 3 days later
    let proposed_price = 3000; // Correct price
    let tolerance = 100; // Allow 100 units tolerance

    // Validate price - should succeed (this method doesn't return a Result)
    client.validate_price(
        &equipment_id,
        &start_date,
        &end_date,
        &proposed_price,
        &tolerance,
    );
}

#[test]
#[should_panic]
fn test_validate_price_outside_tolerance() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date = (env.ledger().timestamp() / 86400) + 1; // Tomorrow in days
    let end_date = start_date + 3; // 3 days later
    let proposed_price = 5000; // Too high
    let tolerance = 100;

    // Validate price - should fail (this method will panic on error)
    client.validate_price(
        &equipment_id,
        &start_date,
        &end_date,
        &proposed_price,
        &tolerance,
    );
}

// ============================================================================
// MAINTENANCE ENFORCEMENT TESTS
// ============================================================================

#[test]
fn test_log_maintenance_success() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let timestamp = env.ledger().timestamp();
    let notes = Some(String::from_str(&env, "Regular maintenance check"));

    // Log maintenance
    client.log_maintenance(
        &equipment_id,
        &MaintenanceStatus::NeedsService,
        &timestamp,
        &notes,
    );

    // Get maintenance history
    let history = client.get_maintenance_history(&Some(equipment_id.clone()));
    assert_eq!(history.len(), 1);

    let record = history.get(0).unwrap();
    assert_eq!(record.equipment_id, equipment_id);
    assert_eq!(record.status, MaintenanceStatus::NeedsService);
    assert_eq!(record.timestamp, timestamp);
    assert_eq!(record.notes, notes);
}

#[test]
fn test_maintenance_history_filtered() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id1 = create_equipment_id(&env, "tractor_001");
    let equipment_id2 = create_equipment_id(&env, "harvester_001");
    let equipment_type = String::from_str(&env, "Agricultural Equipment");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register two pieces of equipment
    client.register_equipment(
        &equipment_id1,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );
    client.register_equipment(
        &equipment_id2,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let timestamp = env.ledger().timestamp();
    let notes = Some(String::from_str(&env, "Maintenance check"));

    // Log maintenance for both equipment
    client.log_maintenance(
        &equipment_id1,
        &MaintenanceStatus::NeedsService,
        &timestamp,
        &notes,
    );
    client.log_maintenance(
        &equipment_id2,
        &MaintenanceStatus::UnderMaintenance,
        &timestamp,
        &notes,
    );

    // Get maintenance history for specific equipment
    let history1 = client.get_maintenance_history(&Some(equipment_id1.clone()));
    assert_eq!(history1.len(), 1);
    assert_eq!(history1.get(0).unwrap().equipment_id, equipment_id1);

    let history2 = client.get_maintenance_history(&Some(equipment_id2.clone()));
    assert_eq!(history2.len(), 1);
    assert_eq!(history2.get(0).unwrap().equipment_id, equipment_id2);

    // Get all maintenance history
    let all_history = client.get_maintenance_history(&None);
    assert_eq!(all_history.len(), 2);
}

#[test]
fn test_maintenance_status_updates() {
    let (env, contract_id, client, _, _, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let timestamp = env.ledger().timestamp();
    let notes = Some(String::from_str(&env, "Status update"));

    // Test all maintenance status transitions
    client.log_maintenance(
        &equipment_id,
        &MaintenanceStatus::NeedsService,
        &timestamp,
        &notes,
    );
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::NeedsService);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(
        equipment.maintenance_status,
        MaintenanceStatus::NeedsService
    );

    client.log_maintenance(
        &equipment_id,
        &MaintenanceStatus::UnderMaintenance,
        &timestamp,
        &notes,
    );
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::UnderMaintenance);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(
        equipment.maintenance_status,
        MaintenanceStatus::UnderMaintenance
    );

    client.log_maintenance(&equipment_id, &MaintenanceStatus::Good, &timestamp, &notes);
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::Good);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::Good);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_rental_lifecycle() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // 1. Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true);
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::Good);

    // 2. Create rental
    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );

    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Pending);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true); // Equipment remains available

    // 3. Confirm rental
    client.confirm_rental(&equipment_id);

    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Active);

    // 4. Complete rental
    client.complete_rental(&equipment_id);

    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Completed);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true);

    // 5. Verify rental history
    let history = client.get_rental_history_by_equipment(&equipment_id);
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().status, RentalStatus::Completed);
}

#[test]
#[should_panic(expected = "Equipment under maintenance or needs service")]
fn test_maintenance_prevents_rental() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id = create_equipment_id(&env, "tractor_001");
    let equipment_type = String::from_str(&env, "Agricultural Tractor");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register equipment
    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    // Set equipment under maintenance
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::UnderMaintenance);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(
        equipment.maintenance_status,
        MaintenanceStatus::UnderMaintenance
    );

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (3 * 86400);
    let total_price = 3000;

    // Try to create rental - should panic
    client.create_rental(
        &equipment_id,
        &renter1,
        &start_date,
        &end_date,
        &total_price,
    );
}

#[test]
fn test_multiple_rentals_same_user() {
    let (env, contract_id, client, _, renter1, _) = setup_test();
    let equipment_id1 = create_equipment_id(&env, "tractor_001");
    let equipment_id2 = create_equipment_id(&env, "harvester_001");
    let equipment_type = String::from_str(&env, "Agricultural Equipment");
    let rental_price_per_day = 1000;
    let location = String::from_str(&env, "Farm Location A");

    // Register two pieces of equipment
    client.register_equipment(
        &equipment_id1,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );
    client.register_equipment(
        &equipment_id2,
        &equipment_type,
        &rental_price_per_day,
        &location,
    );

    let start_date1 = env.ledger().timestamp() + 86400;
    let end_date1 = start_date1 + (3 * 86400);
    let total_price1 = 3000;

    let start_date2 = env.ledger().timestamp() + (10 * 86400);
    let end_date2 = start_date2 + (2 * 86400);
    let total_price2 = 2000;

    // Create rentals for same user on different equipment
    client.create_rental(
        &equipment_id1,
        &renter1,
        &start_date1,
        &end_date1,
        &total_price1,
    );
    client.create_rental(
        &equipment_id2,
        &renter1,
        &start_date2,
        &end_date2,
        &total_price2,
    );

    // Verify user rental history
    let user_history = client.get_rental_history_by_user(&renter1);
    assert_eq!(user_history.len(), 2);

    // Verify both rentals are in user history
    let rental1 = user_history.get(0).unwrap();
    let rental2 = user_history.get(1).unwrap();

    assert_eq!(rental1.equipment_id, equipment_id1);
    assert_eq!(rental2.equipment_id, equipment_id2);
    assert_eq!(rental1.renter, renter1);
    assert_eq!(rental2.renter, renter1);
}
