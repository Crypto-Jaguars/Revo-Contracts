use crate::datatype::{CooperativeError, DataKey, Proposal};
use crate::interface::{Governance, Membership};
use crate::tests::utils::*;
use crate::CooperativeManagementContract;
use soroban_sdk::String;

#[test]
fn test_submit_proposal_success() {
    let test_env = setup_test();

    // Register member
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_member_name(&test_env.env),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit proposal
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    assert!(result.is_ok());

    // Verify proposal
    let proposal = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Proposal>(&DataKey::Proposal(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(proposal.proposer, test_env.member1);
    assert_eq!(proposal.votes_for, 0);
    assert_eq!(proposal.votes_against, 0);
    assert_eq!(proposal.executed, false);
}

#[test]
fn test_submit_proposal_not_a_member() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    assert_eq!(result, Err(CooperativeError::NotAMember));
}

#[test]
fn test_vote_on_proposal_approve() {
    let test_env = setup_test();

    // Register proposer and voter
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Voter"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit proposal
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    // Vote to approve
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    assert!(result.is_ok());

    // Verify vote
    let proposal = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Proposal>(&DataKey::Proposal(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(proposal.votes_for, 1);
    assert_eq!(proposal.votes_against, 0);
}

#[test]
fn test_vote_on_proposal_reject() {
    let test_env = setup_test();

    // Register proposer and voter
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Voter"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit proposal
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    // Vote to reject
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            false,
        )
    });

    assert!(result.is_ok());

    // Verify vote
    let proposal = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Proposal>(&DataKey::Proposal(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(proposal.votes_for, 0);
    assert_eq!(proposal.votes_against, 1);
}

#[test]
fn test_vote_on_proposal_not_a_member() {
    let test_env = setup_test();

    // Register proposer only
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit proposal
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    // Try to vote without being a member
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    assert_eq!(result, Err(CooperativeError::NotAMember));
}

#[test]
fn test_vote_on_proposal_not_found() {
    let test_env = setup_test();

    // Register voter
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Voter"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Try to vote on non-existent proposal
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    assert_eq!(result, Err(CooperativeError::ProposalNotFound));
}

#[test]
fn test_execute_decision_success() {
    let test_env = setup_test();

    // Register members
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Voter1"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member3.clone(),
            String::from_str(&test_env.env, "Voter2"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit proposal
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    // Vote to approve (2 votes for)
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member3.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    // Execute decision
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::execute_decision(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    assert!(result.is_ok());

    // Verify execution
    let proposal = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Proposal>(&DataKey::Proposal(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(proposal.executed, true);
}

#[test]
fn test_execute_decision_insufficient_votes() {
    let test_env = setup_test();

    // Register members
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Voter1"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member3.clone(),
            String::from_str(&test_env.env, "Voter2"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit proposal
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    // Vote: 1 for, 2 against
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member3.clone(),
            test_env.member1.clone(),
            false,
        )
    });

    // Try to execute (should fail)
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::execute_decision(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    assert_eq!(result, Err(CooperativeError::ProposalRejected));
}

#[test]
fn test_execute_decision_already_executed() {
    let test_env = setup_test();

    // Register members
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Voter"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit and approve proposal
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            standard_proposal_description(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    // First execution
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::execute_decision(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    // Try to execute again
    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::execute_decision(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    assert_eq!(result, Err(CooperativeError::ProposalAlreadyExecuted));
}

#[test]
fn test_trigger_emergency_success() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::trigger_emergency(
            test_env.env.clone(),
            test_env.admin.clone(),
            String::from_str(&test_env.env, "Critical system failure"),
        )
    });

    assert!(result.is_ok());

    // Verify emergency is stored
    let emergency_reason = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, String>(&DataKey::Emergency)
    });

    assert!(emergency_reason.is_some());
}

#[test]
fn test_trigger_emergency_unauthorized() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::trigger_emergency(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Unauthorized emergency"),
        )
    });

    assert_eq!(result, Err(CooperativeError::Unauthorized));
}

#[test]
fn test_track_accountability() {
    let test_env = setup_test();

    let result = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::track_accountability(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    // Should return 0 for new member
    assert_eq!(result, Ok(0));
}

#[test]
fn test_proposal_lifecycle() {
    let test_env = setup_test();

    // Register members
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Voter"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Build new storage facility"),
        )
    });

    // Vote
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::vote_on_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            test_env.member1.clone(),
            true,
        )
    });

    // Execute
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::execute_decision(
            test_env.env.clone(),
            test_env.member1.clone(),
        )
    });

    // Verify final state
    let proposal = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Proposal>(&DataKey::Proposal(test_env.member1.clone()))
            .unwrap()
    });

    assert_eq!(proposal.votes_for, 1);
    assert_eq!(proposal.votes_against, 0);
    assert_eq!(proposal.executed, true);
}

#[test]
fn test_multiple_proposals() {
    let test_env = setup_test();

    // Register members (need one per proposal as key is by proposer)
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposer1"),
            standard_farmer_role(&test_env.env),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Membership>::register_member(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Proposer2"),
            standard_farmer_role(&test_env.env),
        )
    });

    // Submit different proposals
    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member1.clone(),
            String::from_str(&test_env.env, "Proposal 1"),
        )
    });

    let _ = test_env.env.as_contract(&test_env.contract_id, || {
        <CooperativeManagementContract as Governance>::submit_proposal(
            test_env.env.clone(),
            test_env.member2.clone(),
            String::from_str(&test_env.env, "Proposal 2"),
        )
    });

    // Verify both exist
    let proposal1 = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Proposal>(&DataKey::Proposal(test_env.member1.clone()))
    });

    let proposal2 = test_env.env.as_contract(&test_env.contract_id, || {
        test_env
            .env
            .storage()
            .persistent()
            .get::<DataKey, Proposal>(&DataKey::Proposal(test_env.member2.clone()))
    });

    assert!(proposal1.is_some());
    assert!(proposal2.is_some());
}
