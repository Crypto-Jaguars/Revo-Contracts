use crate::{CommodityBackedToken, ContractError, Inventory};
use soroban_sdk::{contracttype, Address, BytesN, Env, Map, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    AuthIssuers,
    TokenData(BytesN<32>),
    TokenOwner(BytesN<32>),
    Inventory(String),
    VerificationReg(String),
    CommodityIndex(String),
    TokenNonce,
}

pub fn set_admin(env: &Env, admin: &Address) {
    if env.storage().instance().has(&DataKey::Admin) {
        // If admin is already set, require authorization from current admin
        let current_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        current_admin.require_auth();
    }
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Admin not set")
}

pub fn add_authorized_issuer(
    env: &Env,
    admin: &Address,
    issuer: &Address,
) -> Result<(), ContractError> {
    if *admin != get_admin(env) {
        return Err(ContractError::Unauthorized);
    }

    let key = DataKey::AuthIssuers;
    let mut issuers: Vec<Address> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    if !issuers.iter().any(|x| x == *issuer) {
        issuers.push_back(issuer.clone());
        env.storage().instance().set(&key, &issuers);

        env.events().publish(
            (Symbol::new(env, "issuer_added"), admin.clone()),
            issuer.clone(),
        );
    }

    Ok(())
}

pub fn get_authorized_issuers(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::AuthIssuers)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn store_token(env: &Env, token_id: &BytesN<32>, token: &CommodityBackedToken) {
    let key = DataKey::TokenData(token_id.clone());
    env.storage().instance().set(&key, token);
}

pub fn get_token(env: &Env, token_id: &BytesN<32>) -> Option<CommodityBackedToken> {
    let key = DataKey::TokenData(token_id.clone());
    env.storage().instance().get(&key)
}

pub fn remove_token(env: &Env, token_id: &BytesN<32>) {
    let data_key = DataKey::TokenData(token_id.clone());
    env.storage().instance().remove(&data_key);

    let owner_key = DataKey::TokenOwner(token_id.clone());
    env.storage().instance().remove(&owner_key);
}

pub fn set_token_owner(env: &Env, token_id: &BytesN<32>, owner: &Address) {
    let key = DataKey::TokenOwner(token_id.clone());
    env.storage().instance().set(&key, owner);
}

pub fn get_token_owner(env: &Env, token_id: &BytesN<32>) -> Result<Address, ContractError> {
    let key = DataKey::TokenOwner(token_id.clone());
    env.storage()
        .instance()
        .get(&key)
        .ok_or(ContractError::OwnerNotFound)
}

pub fn get_inventory(env: &Env, commodity_type: &String) -> Inventory {
    let key = DataKey::Inventory(commodity_type.clone());
    env.storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Inventory {
            total_quantity: 0,
            available_quantity: 0,
            issued_tokens: 0,
        })
}

pub fn update_inventory(
    env: &Env,
    commodity_type: &String,
    inventory: &Inventory,
) -> Result<(), ContractError> {
    let key = DataKey::Inventory(commodity_type.clone());
    env.storage().instance().set(&key, inventory);
    Ok(())
}

pub fn add_inventory(
    env: &Env,
    admin: &Address,
    commodity_type: &String,
    quantity: u32,
) -> Result<(), ContractError> {
    if *admin != get_admin(env) {
        return Err(ContractError::Unauthorized);
    }

    admin.require_auth();

    let mut inventory = get_inventory(env, commodity_type);
    inventory.total_quantity = inventory
        .total_quantity
        .checked_add(quantity)
        .ok_or(ContractError::InvalidInput)?;

    inventory.available_quantity = inventory
        .available_quantity
        .checked_add(quantity)
        .ok_or(ContractError::InvalidInput)?;

    update_inventory(env, commodity_type, &inventory)?;

    env.events().publish(
        (Symbol::new(env, "inv_added"), admin.clone()),
        (commodity_type.clone(), quantity),
    );

    Ok(())
}

pub fn get_verification_registry(
    env: &Env,
    commodity_type: &String,
) -> Map<BytesN<32>, Map<String, String>> {
    let key = DataKey::VerificationReg(commodity_type.clone());
    env.storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Map::new(env))
}

pub fn update_verification_registry(
    env: &Env,
    commodity_type: &String,
    registry: &Map<BytesN<32>, Map<String, String>>,
) -> Result<(), ContractError> {
    let key = DataKey::VerificationReg(commodity_type.clone());
    env.storage().instance().set(&key, registry);
    Ok(())
}
