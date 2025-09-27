#![cfg(test)]

extern crate std;

use crate::equipment::MaintenanceStatus;
use soroban_sdk::String;

use super::utils::{register_basic_equipment, setup_test, create_standard_rental};

// ============================================================================
// EQUIPMENT AVAILABILITY TESTS
// ============================================================================

#[test]
fn test_update_equipment_availability() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Initially should be available
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert!(equipment.available);

    // Set unavailable
    client.update_availability(&equipment_id, &false);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert!(!equipment.available);

    // Set back to available
    client.update_availability(&equipment_id, &true);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert!(equipment.available);
}

#[test]
fn test_equipment_registration_defaults() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.available, true); // Default availability
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::Good); // Default status
    assert_eq!(equipment.rental_price_per_day, 1000);
}

#[test]
fn test_get_nonexistent_equipment() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = super::utils::create_equipment_id(&env, "nonexistent");

    let equipment = client.get_equipment(&equipment_id);
    assert!(equipment.is_none());
}

// ============================================================================
// MAINTENANCE STATUS TESTS
// ============================================================================

#[test]
fn test_maintenance_status_transitions() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Test all status transitions
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::NeedsService);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::NeedsService);

    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::UnderMaintenance);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::UnderMaintenance);

    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::Good);
    let equipment = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(equipment.maintenance_status, MaintenanceStatus::Good);
}

#[test]
#[should_panic(expected = "Equipment under maintenance or needs service")]
fn test_maintenance_blocks_rental_creation() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Set to NeedsService
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::NeedsService);

    let start_date = env.ledger().timestamp() + 86400;
    let end_date = start_date + (2 * 86400);
    let total_price = 2000;

    // Should panic when trying to create rental
    client.create_rental(&equipment_id, &renter1, &start_date, &end_date, &total_price);
}

#[test]
fn test_maintenance_allows_rental_after_fixed() {
    let (env, _contract_id, client, _owner, renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Set to maintenance
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::UnderMaintenance);

    // Fix and set back to Good
    client.update_maintenance_status(&equipment_id, &MaintenanceStatus::Good);

    // Should now allow rental creation
    create_standard_rental(&client, &env, &equipment_id, &renter1, 2);
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.renter, renter1);
}

// ============================================================================
// SCHEDULING CONFLICT TESTS
// ============================================================================

#[test]
#[should_panic(expected = "Rental already exists for this equipment")]
fn test_scheduling_conflict_with_pending_rental() {
    let (env, _contract_id, client, _owner, renter1, renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Create first rental (Pending)
    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);

    // Try to create overlapping rental - should fail
    create_standard_rental(&client, &env, &equipment_id, &renter2, 2);
}

#[test]
#[should_panic(expected = "Rental already exists for this equipment")]
fn test_scheduling_conflict_with_active_rental() {
    let (env, _contract_id, client, _owner, renter1, renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Create and confirm first rental (Active)
    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.confirm_rental(&equipment_id);

    // Try to create new rental while active - should fail
    let start_date = env.ledger().timestamp() + (5 * 86400);
    let end_date = start_date + (2 * 86400);
    let total_price = 2000;
    
    client.create_rental(&equipment_id, &renter2, &start_date, &end_date, &total_price);
}

#[test]
fn test_scheduling_after_rental_completion() {
    let (env, _contract_id, client, _owner, renter1, renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Create, confirm, and complete first rental
    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.confirm_rental(&equipment_id);
    client.complete_rental(&equipment_id);

    // Should be able to create new rental after completion
    let start_date = env.ledger().timestamp() + (10 * 86400);
    let end_date = start_date + (2 * 86400);
    let total_price = 2000;
    
    client.create_rental(&equipment_id, &renter2, &start_date, &end_date, &total_price);
    
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.renter, renter2);
    assert_eq!(rental.total_price, total_price);
}

#[test]
fn test_scheduling_after_rental_cancellation() {
    let (env, _contract_id, client, _owner, renter1, renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // Create and cancel first rental
    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);
    client.cancel_rental(&equipment_id);

    // Should be able to create new rental after cancellation
    let start_date = env.ledger().timestamp() + (10 * 86400);
    let end_date = start_date + (2 * 86400);
    let total_price = 2000;
    
    client.create_rental(&equipment_id, &renter2, &start_date, &end_date, &total_price);
    
    let rental = client.get_rental(&equipment_id).unwrap();
    assert_eq!(rental.renter, renter2);
    assert_eq!(rental.total_price, total_price);
}

// ============================================================================
// MAINTENANCE HISTORY TESTS
// ============================================================================

#[test]
fn test_maintenance_logging() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    let timestamp = env.ledger().timestamp();
    let notes = Some(String::from_str(&env, "Regular maintenance check"));

    client.log_maintenance(&equipment_id, &MaintenanceStatus::NeedsService, &timestamp, &notes);

    let history = client.get_maintenance_history(&Some(equipment_id.clone()));
    assert_eq!(history.len(), 1);

    let record = history.get(0).unwrap();
    assert_eq!(record.equipment_id, equipment_id);
    assert_eq!(record.status, MaintenanceStatus::NeedsService);
    assert_eq!(record.timestamp, timestamp);
    assert_eq!(record.notes, notes);
}

#[test]
fn test_maintenance_history_filtering() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    let equipment_id1 = register_basic_equipment(&client, &env, "tractor_001", 1000);
    let equipment_id2 = register_basic_equipment(&client, &env, "harvester_001", 1500);

    let timestamp = env.ledger().timestamp();

    // Log maintenance for both equipment
    client.log_maintenance(&equipment_id1, &MaintenanceStatus::NeedsService, &timestamp, &None);
    client.log_maintenance(&equipment_id2, &MaintenanceStatus::UnderMaintenance, &timestamp, &None);

    // Test filtered history
    let history1 = client.get_maintenance_history(&Some(equipment_id1.clone()));
    assert_eq!(history1.len(), 1);
    assert_eq!(history1.get(0).unwrap().equipment_id, equipment_id1);

    let history2 = client.get_maintenance_history(&Some(equipment_id2.clone()));
    assert_eq!(history2.len(), 1);
    assert_eq!(history2.get(0).unwrap().equipment_id, equipment_id2);

    // Test unfiltered history
    let all_history = client.get_maintenance_history(&None);
    assert_eq!(all_history.len(), 2);
}

// ============================================================================
// HIGH-VOLUME AVAILABILITY TESTS
// ============================================================================

#[test]
fn test_multiple_equipment_availability() {
    let (env, _contract_id, client, _owner, _renter1, _renter2) = setup_test();
    
    // Register multiple pieces of equipment
    let eq1 = register_basic_equipment(&client, &env, "equipment_0", 1000);
    let eq2 = register_basic_equipment(&client, &env, "equipment_1", 1100);
    let eq3 = register_basic_equipment(&client, &env, "equipment_2", 1200);

    // Test setting different availability states
    client.update_availability(&eq1, &true);
    client.update_availability(&eq2, &false);
    client.update_availability(&eq3, &true);
        
    let equipment1 = client.get_equipment(&eq1).unwrap();
    let equipment2 = client.get_equipment(&eq2).unwrap();
    let equipment3 = client.get_equipment(&eq3).unwrap();
    
    assert_eq!(equipment1.available, true);
    assert_eq!(equipment2.available, false);
    assert_eq!(equipment3.available, true);
}

#[test]
#[should_panic(expected = "Rental already exists for this equipment")]
fn test_concurrent_rental_attempts() {
    let (env, _contract_id, client, _owner, renter1, renter2) = setup_test();
    let equipment_id = register_basic_equipment(&client, &env, "tractor_001", 1000);

    // First renter creates rental
    create_standard_rental(&client, &env, &equipment_id, &renter1, 3);

    // Second renter attempt should fail
    create_standard_rental(&client, &env, &equipment_id, &renter2, 2);
}
