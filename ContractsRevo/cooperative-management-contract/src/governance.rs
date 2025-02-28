use crate::CooperativeManagementContract;
use crate::datatype::{CooperativeError, DataKey, Proposal};
use crate::interface::Governance;
use soroban_sdk::{Address, Env, String};

impl Governance for CooperativeManagementContract {
    fn submit_proposal(
        env: Env,
        proposer: Address,
        description: String,
    ) -> Result<(), CooperativeError> {
        let member_key = DataKey::Member(proposer.clone());
        // Check if the proposer is a registered member
        if !env.storage().persistent().has(&member_key) {
            return Err(CooperativeError::NotAMember);
        }

        let key = DataKey::Proposal(proposer.clone());
        let proposal = Proposal {
            proposer: proposer.clone(),
            description,
            votes_for: 0,
            votes_against: 0,
            executed: false,
        };

        env.storage().persistent().set(&key, &proposal);
        Ok(())
    }

    fn vote_on_proposal(
        env: Env,
        voter: Address,
        proposer: Address,
        approve: bool,
    ) -> Result<(), CooperativeError> {
        let member_key = DataKey::Member(voter.clone());
        // Check if the voter is a registered member
        if !env.storage().persistent().has(&member_key) {
            return Err(CooperativeError::NotAMember);
        }

        let key = DataKey::Proposal(proposer.clone());
        if let Some(mut proposal) = env.storage().persistent().get::<DataKey, Proposal>(&key) {
            if approve {
                proposal.votes_for += 1;
            } else {
                proposal.votes_against += 1;
            }
            env.storage().persistent().set(&key, &proposal);
            Ok(())
        } else {
            Err(CooperativeError::ProposalNotFound)
        }
    }

    fn execute_decision(env: Env, proposer: Address) -> Result<(), CooperativeError> {
        let key = DataKey::Proposal(proposer.clone());

        if let Some(mut proposal) = env.storage().persistent().get::<DataKey, Proposal>(&key) {
            if proposal.executed {
                return Err(CooperativeError::ProposalAlreadyExecuted);
            }

            if proposal.votes_for > proposal.votes_against {
                proposal.executed = true;
                env.storage().persistent().set(&key, &proposal);
                Ok(())
            } else {
                Err(CooperativeError::ProposalRejected)
            }
        } else {
            Err(CooperativeError::ProposalNotFound)
        }
    }

    fn trigger_emergency(env: Env, caller: Address, reason: String) -> Result<(), CooperativeError> {
        let admin_key = DataKey::Admin; // Assuming Admin address is stored
        let admin = env.storage().persistent().get::<DataKey, Address>(&admin_key);
    
        if Some(caller.clone()) != admin {
            return Err(CooperativeError::Unauthorized);
        }
    
        let key = DataKey::Emergency;
        env.storage().persistent().set(&key, &reason);
        Ok(())
    }
    

    fn track_accountability(env: Env, member: Address) -> Result<i128, CooperativeError> {
        let key = DataKey::Reputation(member.clone());
        let reputation = env
            .storage()
            .persistent()
            .get::<DataKey, i128>(&key)
            .unwrap_or(0);
        Ok(reputation)
    }
}
