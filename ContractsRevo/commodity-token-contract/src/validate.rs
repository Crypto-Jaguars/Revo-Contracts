use soroban_sdk::{BytesN, Env, String, Map, Address, Symbol};
use crate::{storage, ContractError};

pub fn validate_commodity(
    env: &Env,
    commodity_type: &String,
    verification_data: &BytesN<32>,
) -> bool {
    let verification_registry = storage::get_verification_registry(env, commodity_type);

    verification_registry.contains_key(verification_data.clone())
}

pub fn register_commodity_verification(
    env: &Env,
    admin: &Address,
    commodity_type: &String,
    verification_data: &BytesN<32>,
    metadata: &Map<String, String>,
) -> Result<(), ContractError> {
    let stored_admin = storage::get_admin(env);
    if stored_admin != *admin {
        return Err(ContractError::Unauthorized);
    }
    admin.require_auth();

    let mut registry = storage::get_verification_registry(env, commodity_type);

    registry.set(verification_data.clone(), metadata.clone());

    storage::update_verification_registry(env, commodity_type, &registry)?;

    env.events().publish(
        (Symbol::new(env, "verification_registered"), admin.clone()),
        (commodity_type.clone(), verification_data.clone()),
    );
    
    Ok(())
}

pub fn check_expiration(
    env: &Env,
    token_id: &BytesN<32>,
) -> bool {
    if let Some(token) = storage::get_token(env, token_id) {
        let current_time = env.ledger().timestamp();
        return current_time <= token.expiration_date;
    }
    false
}