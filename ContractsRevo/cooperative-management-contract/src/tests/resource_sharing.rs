use crate::datatype::{CooperativeError, DataKey, Resource};
use crate::interface::{Membership, ResourceSharing};
use crate::tests::utils::*;
use crate::CooperativeManagementContract;
use soroban_sdk::{String, Vec};

#[test]
fn test_register_resource_success() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    assert!(result.is_ok());

    // Verify resource was registered
    let resources = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::get_resources_by_owner(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    assert_eq!(resources.len(), 1);
}

#[test]
fn test_register_multiple_resources() {
    let test_env = setup_test();

    // Register three resources
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Tractor"),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Harvester"),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Warehouse"),
        )
    });

    // Verify all resources
    let resources = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::get_resources_by_owner(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    assert_eq!(resources.len(), 3);
}

#[test]
fn test_borrow_resource_success() {
    let test_env = setup_test();

    // Register member as borrower
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Borrow resource
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::borrow_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    assert!(result.is_ok());

    // Verify resource is borrowed
    let resource = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&DataKey::Resource(test_env.member1.clone(), 1))
            .unwrap()
    });

    assert_eq!(resource.available, false);
    assert_eq!(resource.borrower, Some(test_env.member2.clone()));
}

#[test]
fn test_borrow_resource_not_a_member() {
    let test_env = setup_test();

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Try to borrow without being a member
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::borrow_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    assert_eq!(result, Err(CooperativeError::NotAMember));
}

#[test]
fn test_borrow_resource_double_booking() {
    let test_env = setup_test();

    // Register two members
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower1"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member3.clone(),
            String::from_str(&test_env.env, "Borrower2"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // First borrow
    let result1 = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::borrow_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });
    assert!(result1.is_ok());

    // Second borrow attempt (should fail - double booking)
    let result2 = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::borrow_resource(
            test_env.env.clone(),
            test_env.member3.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    assert_eq!(result2, Err(CooperativeError::ResourceNotAvailable));
}

#[test]
fn test_return_resource_success() {
    let test_env = setup_test();

    // Register borrower
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register and borrow resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::borrow_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    // Return resource
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::return_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    assert!(result.is_ok());

    // Verify resource is available again
    let resource = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&DataKey::Resource(test_env.member1.clone(), 1))
            .unwrap()
    });

    assert_eq!(resource.available, true);
    assert_eq!(resource.borrower, None);
}

#[test]
fn test_return_resource_unauthorized() {
    let test_env = setup_test();

    // Register two members
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member3.clone(),
            String::from_str(&test_env.env, "Other"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register and borrow resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::borrow_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    // Try to return by unauthorized member
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::return_resource(
            test_env.env.clone(),
            test_env.member3.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    assert_eq!(result, Err(CooperativeError::Unauthorized));
}

#[test]
fn test_schedule_resource_success() {
    let test_env = setup_test();

    // Register borrower
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Schedule resource
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::schedule_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            1,
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Monday 9-12"),
        )
    });

    assert!(result.is_ok());

    // Verify schedule
    let resource = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&DataKey::Resource(test_env.member1.clone(), 1))
            .unwrap()
    });

    assert_eq!(resource.schedule.len(), 1);
}

#[test]
fn test_schedule_resource_conflict() {
    let test_env = setup_test();

    // Register borrowers
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower1"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member3.clone(),
            String::from_str(&test_env.env, "Borrower2"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // First schedule
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::schedule_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            1,
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Monday 9-12"),
        )
    });

    // Second schedule with conflict
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::schedule_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            1,
            test_env.member3.clone(),
            String::from_str(&test_env.env, "Monday 9-12"),
        )
    });

    assert_eq!(result, Err(CooperativeError::TimeSlotConflict));
}

#[test]
fn test_schedule_resource_multiple_slots() {
    let test_env = setup_test();

    // Register borrower
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Schedule multiple time slots
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::schedule_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            1,
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Monday 9-12"),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::schedule_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            1,
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Tuesday 14-17"),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::schedule_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            1,
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Friday 10-13"),
        )
    });

    // Verify schedules
    let resource = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&DataKey::Resource(test_env.member1.clone(), 1))
            .unwrap()
    });

    assert_eq!(resource.schedule.len(), 3);
}

#[test]
fn test_track_maintenance_success() {
    let test_env = setup_test();

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Track maintenance
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::track_maintenance(
            test_env.env.clone(),
            test_env.member1.clone(),
            test_env.member1.clone(),
            1,
            String::from_str(&test_env.env, "Oil change completed"),
        )
    });

    assert!(result.is_ok());

    // Verify maintenance log
    let logs = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Vec<String>>(&DataKey::MaintenanceLog(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(logs.len(), 1);
}

#[test]
fn test_track_maintenance_unauthorized() {
    let test_env = setup_test();

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Try to track maintenance by non-owner
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::track_maintenance(
            test_env.env.clone(),
            test_env.member1.clone(),
            test_env.member2.clone(),
            1,
            String::from_str(&test_env.env, "Unauthorized maintenance"),
        )
    });

    assert_eq!(result, Err(CooperativeError::Unauthorized));
}

#[test]
fn test_track_maintenance_resource_not_found() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::track_maintenance(
            test_env.env.clone(),
            test_env.member1.clone(),
            test_env.member1.clone(),
            99,
            String::from_str(&test_env.env, "Maintenance"),
        )
    });

    assert_eq!(result, Err(CooperativeError::ResourceNotFound));
}

#[test]
fn test_track_maintenance_multiple_records() {
    let test_env = setup_test();

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Track multiple maintenance records
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::track_maintenance(
            test_env.env.clone(),
            test_env.member1.clone(),
            test_env.member1.clone(),
            1,
            String::from_str(&test_env.env, "Oil change"),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::track_maintenance(
            test_env.env.clone(),
            test_env.member1.clone(),
            test_env.member1.clone(),
            1,
            String::from_str(&test_env.env, "Tire replacement"),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::track_maintenance(
            test_env.env.clone(),
            test_env.member1.clone(),
            test_env.member1.clone(),
            1,
            String::from_str(&test_env.env, "Annual inspection"),
        )
    });

    // Verify all logs
    let logs = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Vec<String>>(&DataKey::MaintenanceLog(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(logs.len(), 3);
}

#[test]
fn test_resource_lifecycle_workflow() {
    let test_env = setup_test();

    // Register borrower
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Borrower"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Register resource
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::register_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_resource_description(&test_env.env),
        )
    });

    // Schedule
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::schedule_resource(
            test_env.env.clone(),
            test_env.member1.clone(),
            1,
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Monday 9-12"),
        )
    });

    // Borrow
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::borrow_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    // Return
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::return_resource(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            1,
        )
    });

    // Track maintenance
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as ResourceSharing>::track_maintenance(
            test_env.env.clone(),
            test_env.member1.clone(),
            test_env.member1.clone(),
            1,
            String::from_str(&test_env.env, "Post-use inspection"),
        )
    });

    // Verify final state
    let resource = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&DataKey::Resource(test_env.member1.clone(), 1))
            .unwrap()
    });

    assert_eq!(resource.available, true);
    assert_eq!(resource.borrower, None);
    assert_eq!(resource.schedule.len(), 1);
}
