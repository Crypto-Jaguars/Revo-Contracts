use soroban_sdk::{BytesN, Env, String, Map, Vec, Val, IntoVal};
use crate::{CommodityBackedToken, storage, ContractError};
use crate::storage::DataKey;
use crate::validate;

pub fn get_token_metadata(env: &Env, token_id: &BytesN<32>) -> Result<CommodityBackedToken, ContractError> {
    storage::get_token(env, token_id)
        .ok_or(ContractError::TokenNotFound)
}

pub fn get_token_details(
    env: &Env,
    token_id: &BytesN<32>,
) -> Result<Map<String, Val>, ContractError> {
    let token = get_token_metadata(env, token_id)?;

    let mut details = Map::new(env);

    details.set(
        String::from_str(env, "commodity_type"),
        token.commodity_type.into_val(env),
    );
    details.set(
        String::from_str(env, "quantity"),
        token.quantity.into_val(env),
    );
    details.set(
        String::from_str(env, "grade"),
        token.grade.into_val(env),
    );
    details.set(
        String::from_str(env, "storage_location"),
        token.storage_location.into_val(env),
    );
    details.set(
        String::from_str(env, "expiration_date"),
        token.expiration_date.into_val(env),
    );

    let is_valid = validate::check_expiration(env, token_id);
    details.set(
        String::from_str(env, "valid"),
        is_valid.into_val(env),
    );

    Ok(details)
}

pub fn list_tokens_by_commodity(
    env: &Env,
    commodity_type: &String,
) -> Vec<BytesN<32>> {
    let key = DataKey::CommodityIndex(commodity_type.clone());
    env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env))
}

pub fn add_to_commodity_index(
    env: &Env,
    commodity_type: &String,
    token_id: &BytesN<32>,
) {
    let key = DataKey::CommodityIndex(commodity_type.clone());
    let mut token_ids: Vec<BytesN<32>> = env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env));

    if !token_ids.iter().any(|id| &id == token_id) {
        token_ids.push_back(token_id.clone());
        env.storage().instance().set(&key, &token_ids);
    }
}

pub fn remove_from_commodity_index(
    env: &Env,
    commodity_type: &String,
    token_id: &BytesN<32>,
) {
    let key = DataKey::CommodityIndex(commodity_type.clone());
    let token_ids_opt: Option<Vec<BytesN<32>>> = env.storage().instance().get(&key);

    if let Some(token_ids) = token_ids_opt {
        let mut updated_ids = Vec::new(env);
        let mut changed = false;
        for id in token_ids.iter() {
            if &id == token_id {
                changed = true;
            } else {
                updated_ids.push_back(id.clone());
            }
        }
        if changed {
            if updated_ids.is_empty() {
                env.storage().instance().remove(&key);
            } else {
                env.storage().instance().set(&key, &updated_ids);
            }
        }
    }
}
