#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Env, String};

// Test helper function to create a new environment
fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    
    // Initialize contract with admin
    let contract = CooperativeManagementContract;
    contract.init(&env, &admin);
    
    (env, admin, member1, member2)
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract = CooperativeManagementContract;
    
    // Test successful initialization
    contract.init(&env, &admin);
    
    // Test double initialization (should fail)
    let result = std::panic::catch_unwind(|| {
        contract.init(&env, &admin);
    });
    assert!(result.is_err());
}

#[test]
fn test_member_registration_validation() {
    let (env, admin, member1, _) = setup();
    let contract = CooperativeManagementContract;
    
    // Test successful registration with valid data
    assert!(contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).is_ok());
    
    // Test registration with empty name
    let empty_name = String::from_str(&env, "");
    assert!(contract.register_member(&env, &member1, empty_name).is_err());
    
    // Test duplicate registration
    assert!(contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).is_err());
}

#[test]
fn test_member_verification_authorization() {
    let (env, admin, member1, _) = setup();
    let contract = CooperativeManagementContract;
    
    // Register member
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    
    // Test verification by admin (should succeed)
    assert!(contract.verify_member(&env, &admin, &member1).is_ok());
    
    // Test verification by non-admin (should fail)
    let non_admin = Address::generate(&env);
    assert!(contract.verify_member(&env, &non_admin, &member1).is_err());
    
    // Test verification of non-existent member
    let non_existent = Address::generate(&env);
    assert!(contract.verify_member(&env, &admin, &non_existent).is_err());
}

#[test]
fn test_membership_levels() {
    let (env, admin, member1, _) = setup();
    let contract = CooperativeManagementContract;
    
    // Register member
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    
    // Verify member (basic level)
    assert!(contract.verify_member(&env, &admin, &member1).is_ok());
    
    // Track contributions to increase level
    assert!(contract.track_contribution(&env, &member1, 100).is_ok());
    
    // Update reputation to reflect higher level
    assert!(contract.update_reputation(&env, &admin, &member1, 10).is_ok());
    
    // Check accountability reflects the new level
    let reputation = contract.track_accountability(&env, &member1).unwrap();
    assert!(reputation > 0);
}

#[test]
fn test_resource_sharing_validation() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Test resource registration
    assert!(contract.register_resource(
        &env,
        &member1,
        String::from_str(&env, "Tractor")
    ).is_ok());
    
    // Test resource registration with empty description
    assert!(contract.register_resource(
        &env,
        &member1,
        String::from_str(&env, "")
    ).is_err());
    
    // Test borrowing by non-member
    let non_member = Address::generate(&env);
    assert!(contract.borrow_resource(&env, &non_member, &member1, 1).is_err());
    
    // Test borrowing already borrowed resource
    assert!(contract.borrow_resource(&env, &member2, &member1, 1).is_ok());
    assert!(contract.borrow_resource(&env, &member2, &member1, 1).is_err());
}

#[test]
fn test_resource_scheduling_conflicts() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Register resource
    assert!(contract.register_resource(
        &env,
        &member1,
        String::from_str(&env, "Tractor")
    ).is_ok());
    
    // Schedule resource for member2
    assert!(contract.schedule_resource(
        &env,
        &member1,
        1,
        &member2,
        String::from_str(&env, "2024-03-20 10:00")
    ).is_ok());
    
    // Try to schedule the same time slot (should fail)
    let member3 = Address::generate(&env);
    contract.register_member(&env, &member3, String::from_str(&env, "Member 3")).unwrap();
    assert!(contract.schedule_resource(
        &env,
        &member1,
        1,
        &member3,
        String::from_str(&env, "2024-03-20 10:00")
    ).is_err());
    
    // Schedule a different time slot (should succeed)
    assert!(contract.schedule_resource(
        &env,
        &member1,
        1,
        &member3,
        String::from_str(&env, "2024-03-20 14:00")
    ).is_ok());
}

#[test]
fn test_equitable_resource_distribution() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Register multiple resources
    assert!(contract.register_resource(
        &env,
        &member1,
        String::from_str(&env, "Tractor")
    ).is_ok());
    
    assert!(contract.register_resource(
        &env,
        &member1,
        String::from_str(&env, "Harvester")
    ).is_ok());
    
    // Get resources by owner
    let resources = contract.get_resources_by_owner(&env, &member1);
    assert_eq!(resources.len(), 2);
    
    // Borrow resources by different members
    assert!(contract.borrow_resource(&env, &member2, &member1, 1).is_ok());
    
    // Verify resource is no longer available
    let non_member = Address::generate(&env);
    assert!(contract.borrow_resource(&env, &non_member, &member1, 1).is_err());
    
    // Return resource
    assert!(contract.return_resource(&env, &member2, &member1, 1).is_ok());
    
    // Verify resource is available again
    assert!(contract.borrow_resource(&env, &non_member, &member1, 1).is_ok());
}

#[test]
fn test_shared_transportation_coordination() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Register transportation resource
    assert!(contract.register_resource(
        &env,
        &member1,
        String::from_str(&env, "Transport Truck")
    ).is_ok());
    
    // Schedule transportation for member2
    assert!(contract.schedule_resource(
        &env,
        &member1,
        1,
        &member2,
        String::from_str(&env, "2024-03-20 10:00")
    ).is_ok());
    
    // Verify transportation is scheduled
    let resources = contract.get_resources_by_owner(&env, &member1);
    assert_eq!(resources.len(), 1);
    
    // Track maintenance for transportation
    assert!(contract.track_maintenance(
        &env,
        &member1,
        &member1,
        1,
        String::from_str(&env, "Regular maintenance for transport truck")
    ).is_ok());
}

#[test]
fn test_governance_validation() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Test proposal submission by non-member
    let non_member = Address::generate(&env);
    assert!(contract.submit_proposal(
        &env,
        &non_member,
        String::from_str(&env, "Invalid proposal")
    ).is_err());
    
    // Test valid proposal submission
    assert!(contract.submit_proposal(
        &env,
        &member1,
        String::from_str(&env, "Valid proposal")
    ).is_ok());
    
    // Test voting by non-member
    assert!(contract.vote_on_proposal(&env, &non_member, &member1, true).is_err());
    
    // Test voting on non-existent proposal
    assert!(contract.vote_on_proposal(&env, &member2, &non_member, true).is_err());
}

#[test]
fn test_voting_mechanism_integrity() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Submit proposal
    assert!(contract.submit_proposal(
        &env,
        &member1,
        String::from_str(&env, "New resource acquisition")
    ).is_ok());
    
    // Vote for proposal
    assert!(contract.vote_on_proposal(&env, &member2, &member1, true).is_ok());
    
    // Try to vote again (should fail)
    assert!(contract.vote_on_proposal(&env, &member2, &member1, false).is_err());
    
    // Execute decision
    assert!(contract.execute_decision(&env, &member1).is_ok());
    
    // Try to execute again (should fail)
    assert!(contract.execute_decision(&env, &member1).is_err());
}

#[test]
fn test_decision_execution_logic() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Submit proposal
    assert!(contract.submit_proposal(
        &env,
        &member1,
        String::from_str(&env, "New resource acquisition")
    ).is_ok());
    
    // Vote against proposal
    assert!(contract.vote_on_proposal(&env, &member2, &member1, false).is_ok());
    
    // Try to execute decision (should fail due to negative votes)
    assert!(contract.execute_decision(&env, &member1).is_err());
    
    // Submit new proposal
    assert!(contract.submit_proposal(
        &env,
        &member1,
        String::from_str(&env, "Another proposal")
    ).is_ok());
    
    // Vote for proposal
    assert!(contract.vote_on_proposal(&env, &member2, &member1, true).is_ok());
    
    // Execute decision (should succeed)
    assert!(contract.execute_decision(&env, &member1).is_ok());
}

#[test]
fn test_emergency_protocol_authorization() {
    let (env, admin, member1, _) = setup();
    let contract = CooperativeManagementContract;
    
    // Test emergency trigger by non-admin
    assert!(contract.trigger_emergency(
        &env,
        &member1,
        String::from_str(&env, "Unauthorized emergency")
    ).is_err());
    
    // Test emergency trigger by admin
    assert!(contract.trigger_emergency(
        &env,
        &admin,
        String::from_str(&env, "Authorized emergency")
    ).is_ok());
}

#[test]
fn test_maintenance_logging_validation() {
    let (env, admin, member1, _) = setup();
    let contract = CooperativeManagementContract;
    
    // Register member and resource
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_resource(
        &env,
        &member1,
        String::from_str(&env, "Tractor")
    ).unwrap();
    
    // Test maintenance logging by owner
    assert!(contract.track_maintenance(
        &env,
        &member1,
        &member1,
        1,
        String::from_str(&env, "Valid maintenance log")
    ).is_ok());
    
    // Test maintenance logging by non-owner
    let non_owner = Address::generate(&env);
    assert!(contract.track_maintenance(
        &env,
        &member1,
        &non_owner,
        1,
        String::from_str(&env, "Invalid maintenance log")
    ).is_err());
    
    // Test maintenance logging for non-existent resource
    assert!(contract.track_maintenance(
        &env,
        &member1,
        &member1,
        999,
        String::from_str(&env, "Invalid resource ID")
    ).is_err());
}

#[test]
fn test_reputation_and_contribution_validation() {
    let (env, admin, member1, _) = setup();
    let contract = CooperativeManagementContract;
    
    // Register member
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    
    // Test contribution tracking for non-member
    let non_member = Address::generate(&env);
    assert!(contract.track_contribution(&env, &non_member, 100).is_err());
    
    // Test valid contribution tracking
    assert!(contract.track_contribution(&env, &member1, 100).is_ok());
    
    // Test reputation update by non-admin
    assert!(contract.update_reputation(&env, &member1, &member1, 10).is_err());
    
    // Test valid reputation update
    assert!(contract.update_reputation(&env, &admin, &member1, 10).is_ok());
    
    // Test reputation update for non-existent member
    assert!(contract.update_reputation(&env, &admin, &non_member, 10).is_err());
}

#[test]
fn test_decision_traceability() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Submit proposal
    assert!(contract.submit_proposal(
        &env,
        &member1,
        String::from_str(&env, "New resource acquisition")
    ).is_ok());
    
    // Vote for proposal
    assert!(contract.vote_on_proposal(&env, &member2, &member1, true).is_ok());
    
    // Execute decision
    assert!(contract.execute_decision(&env, &member1).is_ok());
    
    // Track accountability for both members
    let member1_accountability = contract.track_accountability(&env, &member1).unwrap();
    let member2_accountability = contract.track_accountability(&env, &member2).unwrap();
    
    // Verify accountability reflects participation
    assert!(member1_accountability >= 0);
    assert!(member2_accountability >= 0);
}

#[test]
fn test_governance_disputes() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Submit proposal
    assert!(contract.submit_proposal(
        &env,
        &member1,
        String::from_str(&env, "Controversial proposal")
    ).is_ok());
    
    // Vote against proposal
    assert!(contract.vote_on_proposal(&env, &member2, &member1, false).is_ok());
    
    // Try to execute decision (should fail due to tie)
    assert!(contract.execute_decision(&env, &member1).is_err());
    
    // Trigger emergency to resolve dispute
    assert!(contract.trigger_emergency(
        &env,
        &admin,
        String::from_str(&env, "Resolving governance dispute")
    ).is_ok());
}

#[test]
fn test_cooperative_participation_metrics() {
    let (env, admin, member1, member2) = setup();
    let contract = CooperativeManagementContract;
    
    // Register members
    contract.register_member(&env, &member1, String::from_str(&env, "Member 1")).unwrap();
    contract.register_member(&env, &member2, String::from_str(&env, "Member 2")).unwrap();
    
    // Track contributions
    assert!(contract.track_contribution(&env, &member1, 100).is_ok());
    assert!(contract.track_contribution(&env, &member2, 50).is_ok());
    
    // Update reputation
    assert!(contract.update_reputation(&env, &admin, &member1, 10).is_ok());
    assert!(contract.update_reputation(&env, &admin, &member2, 5).is_ok());
    
    // Submit and vote on proposals
    assert!(contract.submit_proposal(
        &env,
        &member1,
        String::from_str(&env, "Proposal 1")
    ).is_ok());
    
    assert!(contract.vote_on_proposal(&env, &member2, &member1, true).is_ok());
    
    // Execute decision
    assert!(contract.execute_decision(&env, &member1).is_ok());
    
    // Check accountability metrics
    let member1_accountability = contract.track_accountability(&env, &member1).unwrap();
    let member2_accountability = contract.track_accountability(&env, &member2).unwrap();
    
    // Verify metrics reflect participation
    assert!(member1_accountability > member2_accountability);
} 