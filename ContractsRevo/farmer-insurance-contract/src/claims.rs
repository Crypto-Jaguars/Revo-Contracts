use soroban_sdk::{Env, BytesN, contracttype, symbol_short};
use crate::utils::{DataKey, generate_claim_id, ContractError};
use crate::insurance::InsurancePolicy;

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub claim_id: BytesN<32>,
    pub policy_id: BytesN<32>,
    pub event_hash: BytesN<32>,
    pub payout_amount: i128,
}

pub fn sub_claim(
    env: Env,
    policy_id: BytesN<32>,
    event_hash: BytesN<32>,
    payout_amount: i128,
) -> Result<BytesN<32>, ContractError> {
    let policy = env
        .storage()
        .instance()
        .get::<_, InsurancePolicy>(&DataKey::Policy(policy_id.clone()))
        .unwrap_or_else(|| panic!("Policy not found"));

    policy.farmer.require_auth();

    if !policy.active {
        panic!("Policy is not active");
    }

    let claim_id = generate_claim_id(&env)?;

    let claim = Claim {
        claim_id: claim_id.clone(),
        policy_id,
        event_hash,
        payout_amount,
    };

    env.storage()
        .instance()
        .set(&DataKey::Claim(claim_id.clone()), &claim);
    
    env.events()
        .publish((symbol_short!("CLAIM"), claim_id.clone()), claim);

    Ok(claim_id)
}