use soroban_sdk::{Env, Address, BytesN, Symbol, contracttype, symbol_short};
use crate::utils::{DataKey, generate_policy_id, ContractError};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsurancePolicy {
    pub policy_id: BytesN<32>,
    pub farmer: Address,
    pub coverage: Symbol,
    pub premium: i128,
    pub active: bool,
}

pub fn create_pol(env: Env, farmer: Address, coverage: Symbol, premium: i128) -> Result<BytesN<32>, ContractError> {
    farmer.require_auth();

    let policy_id = generate_policy_id(&env)?;
    let policy = InsurancePolicy {
        policy_id: policy_id.clone(),
        farmer: farmer.clone(),
        coverage,
        premium,
        active: false,
    };

    env.storage().instance().set(&DataKey::Policy(policy_id.clone()), &policy);
    env.events().publish((symbol_short!("POLICY"), policy_id.clone()), policy.clone());
    Ok(policy_id)
}

pub fn pay_prem(env: Env, policy_id: BytesN<32>) {
    let mut policy = env
        .storage()
        .instance()
        .get::<_, InsurancePolicy>(&DataKey::Policy(policy_id.clone()))
        .unwrap_or_else(|| panic!("Policy not found"));

    policy.farmer.require_auth();

    if policy.active {
        panic!("Premium already paid");
    }

    policy.active = true;
    env.storage().instance().set(&DataKey::Policy(policy_id.clone()), &policy);
    env.events().publish((symbol_short!("PREMIUM"), policy_id.clone()), policy.clone());
}

pub fn get_policy(env: Env, policy_id: BytesN<32>) -> InsurancePolicy {
    env.storage()
        .instance()
        .get::<_, InsurancePolicy>(&DataKey::Policy(policy_id))
        .unwrap_or_else(|| panic!("Policy not found"))
}