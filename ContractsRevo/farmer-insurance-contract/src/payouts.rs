use soroban_sdk::{Env, BytesN, Address, symbol_short};
use crate::utils::DataKey;
use crate::claims::Claim;
use crate::insurance::InsurancePolicy;

pub fn pay_out(env: Env, claim_id: BytesN<32>, admin: Address) {
    admin.require_auth();

    let claim: Claim = env
        .storage()
        .instance()
        .get::<_, Claim>(&DataKey::Claim(claim_id.clone()))
        .expect("Claim not found");

    let policy: InsurancePolicy = env
        .storage()
        .instance()
        .get::<_, InsurancePolicy>(&DataKey::Policy(claim.policy_id.clone()))
        .expect("Policy not found");

    if !policy.active {
        panic!("Policy is not active or already closed");
    }

    env.storage().instance().remove(&DataKey::Claim(claim_id.clone()));

    env.events().publish(
        (symbol_short!("PAYOUT"), claim_id, policy.farmer.clone()),
        claim.payout_amount,
    );
}