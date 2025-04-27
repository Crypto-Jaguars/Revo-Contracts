use soroban_sdk::{
    contracttype, symbol_short, Address, BytesN, Env, Map, String, Vec,
};
use crate::{CommodityBackedToken, Inventory};

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
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Admin not set")
}

pub fn add_authorized_issuer(env: &Env, admin: &Address, issuer: &Address) {
    if *admin != get_admin(env) {
        panic!("Only admin can add authorized issuers");
    }

    let key = DataKey::AuthIssuers;
    let mut issuers: Vec<Address> = env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env));

    if !issuers.iter().any(|x| x == *issuer) {
        issuers.push_back(issuer.clone());
        env.storage().instance().set(&key, &issuers);
    }
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

pub fn get_token_owner(env: &Env, token_id: &BytesN<32>) -> Address {
    let key = DataKey::TokenOwner(token_id.clone());
    env.storage()
        .instance()
        .get(&key)
        .expect("Token owner not found")
}

pub fn get_inventory(env: &Env, commodity_type: &String) -> Inventory {
    let key = DataKey::Inventory(commodity_type.clone());
    env.storage().instance().get(&key).unwrap_or_else(|| {
        Inventory {
            total_quantity: 0,
            available_quantity: 0,
            issued_tokens: 0,
        }
    })
}

pub fn update_inventory(env: &Env, commodity_type: &String, inventory: &Inventory) {
    let key = DataKey::Inventory(commodity_type.clone());
    env.storage().instance().set(&key, inventory);
}

pub fn add_inventory(
    env: &Env,
    admin: &Address,
    commodity_type: &String,
    quantity: u32,
) {
    if *admin != get_admin(env) {
        panic!("Only admin can add inventory");
    }

    let mut inventory = get_inventory(env, commodity_type);
    inventory.total_quantity += quantity;
    inventory.available_quantity += quantity;
    update_inventory(env, commodity_type, &inventory);

    env.events().publish(
        (symbol_short!("inv_added"), admin.clone()),
        (commodity_type.clone(), quantity),
    );
}

pub fn get_verification_registry(
    env: &Env,
    commodity_type: &String,
) -> Map<BytesN<32>, Map<String, String>> {
    let key = DataKey::VerificationReg(commodity_type.clone());
    env.storage().instance().get(&key).unwrap_or_else(|| Map::new(env))
}

pub fn update_verification_registry(
    env: &Env,
    commodity_type: &String,
    registry: &Map<BytesN<32>, Map<String, String>>,
) {
    let key = DataKey::VerificationReg(commodity_type.clone());
    env.storage().instance().set(&key, registry);
}