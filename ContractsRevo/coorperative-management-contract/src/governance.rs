use crate::CooperativeManagementContract;
use crate::datatype::{CooperativeError, DataKey};
use crate::interface::Governance;
use soroban_sdk::{Address, Env, String};

impl Governance for CooperativeManagementContract {
    fn submit_proposal(
        env: Env,
        proposer: Address,
        proposal: String,
    ) -> Result<(), CooperativeError> {
        let key = DataKey::Proposal(proposer.clone());
        env.storage().persistent().set(&key, &proposal);
        Ok(())
    }

    fn vote_on_proposal(
        env: Env,
        proposer: Address,
        approve: bool,
    ) -> Result<(), CooperativeError> {
        let key = DataKey::ProposalVotes(proposer.clone());
        let mut votes = env
            .storage()
            .persistent()
            .get::<DataKey, (i128, i128)>(&key)
            .unwrap_or((0, 0));
        if approve {
            votes.0 += 1;
        } else {
            votes.1 += 1;
        }
        env.storage().persistent().set(&key, &votes);
        Ok(())
    }

    fn execute_decision(env: Env, proposer: Address) -> Result<(), CooperativeError> {
        let key = DataKey::ProposalVotes(proposer.clone());
        if let Some((yes_votes, no_votes)) = env
            .storage()
            .persistent()
            .get::<DataKey, (i128, i128)>(&key)
        {
            if yes_votes > no_votes {
                env.storage().persistent().remove(&key);
                return Ok(());
            } else {
                return Err(CooperativeError::ProposalRejected);
            }
        }
        Err(CooperativeError::ProposalNotFound)
    }

    fn trigger_emergency(env: Env, reason: String) -> Result<(), CooperativeError> {
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
