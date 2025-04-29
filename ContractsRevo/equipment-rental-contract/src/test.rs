#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, vec, String as _, Symbol as _};

#[test]
fn test_equipment_registration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EquipmentRentalContract);
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let owner = Address::random(&env);

    // Initialize contract
    client.initialize(&admin);

    // Test equipment registration
    let equipment_metadata = EquipmentMetadata {
        name: String::from_str(&env, "Excavator XL2000"),
        description: String::from_str(&env, "Heavy duty excavator"),
        category: String::from_str(&env, "Heavy Equipment"),
        rental_price: 1000,
        location: Location {
            latitude: 37_123456,  // 37.123456
            longitude: -122_987654, // -122.987654
        },
    };

    let equipment_id = client.register_equipment(&owner, &equipment_metadata);
    assert_eq!(equipment_id, 0);

    // Verify equipment details
    let equipment = client.get_equipment(&equipment_id);
    assert_eq!(equipment.owner, owner);
    assert_eq!(equipment.status, EquipmentStatus::Good);
    assert_eq!(equipment.availability, true);
    assert_eq!(equipment.maintenance_history.len(), 0);
    assert_eq!(equipment.rental_history.len(), 0);

    // Test registration with invalid price
    let invalid_metadata = EquipmentMetadata {
        name: String::from_str(&env, "Invalid Equipment"),
        description: String::from_str(&env, "Test"),
        category: String::from_str(&env, "Test"),
        rental_price: 0,
        location: Location {
            latitude: 0,
            longitude: 0,
        },
    };

    let result = client.try_register_equipment(&owner, &invalid_metadata);
    assert!(result.is_err());
}

#[test]
fn test_rental_lifecycle() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EquipmentRentalContract);
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let owner = Address::random(&env);
    let renter = Address::random(&env);

    // Initialize contract and register equipment
    client.initialize(&admin);
    
    let equipment_metadata = EquipmentMetadata {
        name: String::from_str(&env, "Test Equipment"),
        description: String::from_str(&env, "Test Description"),
        category: String::from_str(&env, "Test Category"),
        rental_price: 1000,
        location: Location {
            latitude: 0,
            longitude: 0,
        },
    };

    let equipment_id = client.register_equipment(&owner, &equipment_metadata);

    // Set up rental dates
    let now = 1000000;
    env.ledger().set(Ledger {
        timestamp: now,
        ..Default::default()
    });

    let start_date = now + 86400; // Start tomorrow
    let end_date = start_date + 86400 * 3; // 3 days rental

    // Create rental
    let rental_id = client.create_rental(&renter, &equipment_id, &start_date, &end_date);
    
    // Verify rental details
    let rental = client.get_rental(&rental_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Pending);
    assert_eq!(rental.total_price, 3000); // 3 days * 1000 per day

    // Activate rental
    client.activate_rental(&rental_id);
    
    let equipment = client.get_equipment(&equipment_id);
    assert_eq!(equipment.availability, false);

    // Complete rental
    client.complete_rental(&rental_id);
    
    let equipment = client.get_equipment(&equipment_id);
    assert_eq!(equipment.availability, true);
    assert_eq!(equipment.rental_history.len(), 1);
}

#[test]
fn test_maintenance_enforcement() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EquipmentRentalContract);
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let owner = Address::random(&env);
    let renter = Address::random(&env);

    // Initialize and register equipment
    client.initialize(&admin);
    
    let equipment_metadata = EquipmentMetadata {
        name: String::from_str(&env, "Test Equipment"),
        description: String::from_str(&env, "Test Description"),
        category: String::from_str(&env, "Test Category"),
        rental_price: 1000,
        location: Location {
            latitude: 0,
            longitude: 0,
        },
    };

    let equipment_id = client.register_equipment(&owner, &equipment_metadata);

    // Set equipment under maintenance
    client.update_equipment_status(
        &equipment_id,
        &EquipmentStatus::UnderMaintenance,
        &String::from_str(&env, "Scheduled maintenance"),
    );

    // Try to create rental (should fail)
    let now = 1000000;
    env.ledger().set(Ledger {
        timestamp: now,
        ..Default::default()
    });

    let start_date = now + 86400;
    let end_date = start_date + 86400;

    let result = client.try_create_rental(&renter, &equipment_id, &start_date, &end_date);
    assert!(result.is_err());

    // Update status to NeedsService
    client.update_equipment_status(
        &equipment_id,
        &EquipmentStatus::NeedsService,
        &String::from_str(&env, "Minor repairs needed"),
    );

    // Try to create rental (should still fail)
    let result = client.try_create_rental(&renter, &equipment_id, &start_date, &end_date);
    assert!(result.is_err());

    // Update status to Good
    client.update_equipment_status(
        &equipment_id,
        &EquipmentStatus::Good,
        &String::from_str(&env, "Maintenance completed"),
    );

    // Now rental should succeed
    let rental_id = client.create_rental(&renter, &equipment_id, &start_date, &end_date);
    assert!(rental_id >= 0);

    // Verify maintenance history
    let equipment = client.get_equipment(&equipment_id);
    assert_eq!(equipment.maintenance_history.len(), 3);
}

#[test]
fn test_rental_cancellation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EquipmentRentalContract);
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let owner = Address::random(&env);
    let renter = Address::random(&env);

    // Initialize and register equipment
    client.initialize(&admin);
    
    let equipment_metadata = EquipmentMetadata {
        name: String::from_str(&env, "Test Equipment"),
        description: String::from_str(&env, "Test Description"),
        category: String::from_str(&env, "Test Category"),
        rental_price: 1000,
        location: Location {
            latitude: 0,
            longitude: 0,
        },
    };

    let equipment_id = client.register_equipment(&owner, &equipment_metadata);

    // Set up rental dates
    let now = 1000000;
    env.ledger().set(Ledger {
        timestamp: now,
        ..Default::default()
    });

    let start_date = now + 86400;
    let end_date = start_date + 86400;

    // Create and cancel rental before start date
    let rental_id = client.create_rental(&renter, &equipment_id, &start_date, &end_date);
    client.cancel_rental(&rental_id);

    let rental = client.get_rental(&rental_id).unwrap();
    assert_eq!(rental.status, RentalStatus::Cancelled);

    // Try to cancel after start date (should fail)
    let rental_id2 = client.create_rental(&renter, &equipment_id, &start_date, &end_date);
    
    env.ledger().set(Ledger {
        timestamp: start_date,
        ..Default::default()
    });

    let result = client.try_cancel_rental(&rental_id2);
    assert!(result.is_err());
}

#[test]
fn test_edge_cases() {
    let env = Env::default();
    let contract_id = env.register_contract(None, EquipmentRentalContract);
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    let admin = Address::random(&env);
    let owner = Address::random(&env);
    let renter1 = Address::random(&env);
    let renter2 = Address::random(&env);

    // Initialize and register equipment
    client.initialize(&admin);
    
    let equipment_metadata = EquipmentMetadata {
        name: String::from_str(&env, "Test Equipment"),
        description: String::from_str(&env, "Test Description"),
        category: String::from_str(&env, "Test Category"),
        rental_price: 1000,
        location: Location {
            latitude: 0,
            longitude: 0,
        },
    };

    let equipment_id = client.register_equipment(&owner, &equipment_metadata);

    // Test back-to-back rentals
    let now = 1000000;
    env.ledger().set(Ledger {
        timestamp: now,
        ..Default::default()
    });

    let start_date1 = now + 86400;
    let end_date1 = start_date1 + 86400;
    let start_date2 = end_date1;  // Second rental starts right after first
    let end_date2 = start_date2 + 86400;

    // Create first rental
    let rental_id1 = client.create_rental(&renter1, &equipment_id, &start_date1, &end_date1);
    
    // Create second rental
    let rental_id2 = client.create_rental(&renter2, &equipment_id, &start_date2, &end_date2);

    // Activate first rental
    client.activate_rental(&rental_id1);

    // Try to activate second rental while first is active (should fail)
    let result = client.try_activate_rental(&rental_id2);
    assert!(result.is_err());

    // Complete first rental
    client.complete_rental(&rental_id1);

    // Now second rental can be activated
    let result = client.try_activate_rental(&rental_id2);
    assert!(result.is_ok());
}
