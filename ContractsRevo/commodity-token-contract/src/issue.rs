use soroban_sdk::{
    panic_with_error, symbol_short, Address, Bytes, BytesN, Env, String,
};

extern crate alloc;
use alloc::vec::Vec as StdVec;

use crate::{storage, validate, metadata, CommodityBackedToken};
use crate::storage::DataKey;

#[derive(Debug)]
#[repr(u32)]
pub enum IssueError {
    UnauthorizedIssuer = 1,
    InvalidCommodityData = 2,
    InsufficientInventory = 3,
    InventoryUnderflow = 4,
    InventoryOverflow = 5,
    IdGenerationError = 6,
    InvalidExpirationDate = 7,
    NonceOverflow = 8,
}

impl From<IssueError> for soroban_sdk::Error {
    fn from(err: IssueError) -> Self {
        soroban_sdk::Error::from_contract_error(err as u32)
    }
}

pub fn issue_token(
    env: &Env,
    issuer: &Address,
    commodity_type: &String,
    quantity: u32,
    grade: &String,
    storage_location: &String,
    expiration_date: u64,
    verification_data: &BytesN<32>,
) -> BytesN<32> {

    validate_issuer(env, issuer);

    if !validate::validate_commodity(env, commodity_type, verification_data) {
        panic_with_error!(env, IssueError::InvalidCommodityData);
    }
    let current_time = env.ledger().timestamp();
    if expiration_date <= current_time {
        panic_with_error!(env, IssueError::InvalidExpirationDate);
    }
    let mut inventory = storage::get_inventory(env, commodity_type);
    if inventory.available_quantity < quantity {
        panic_with_error!(env, IssueError::InsufficientInventory);
    }
    
    let nonce_key = DataKey::TokenNonce;
    let current_nonce: u64 = env.storage().instance().get(&nonce_key).unwrap_or(0u64);
    let next_nonce = current_nonce.checked_add(1)
        .unwrap_or_else(|| panic_with_error!(env, IssueError::NonceOverflow));
    env.storage().instance().set(&nonce_key, &next_nonce);


    let token = CommodityBackedToken {
        commodity_type: commodity_type.clone(),
        quantity,
        grade: grade.clone(),
        storage_location: storage_location.clone(),
        expiration_date,
        verification_data: verification_data.clone(),
    };

    let token_id = generate_token_id(
        env,
        quantity,
        expiration_date,
        verification_data,
        current_time,
        current_nonce 
    );

    storage::store_token(env, &token_id, &token);
    
    inventory.available_quantity = inventory.available_quantity.checked_sub(quantity)
        .unwrap_or_else(|| panic_with_error!(env, IssueError::InventoryUnderflow));
    inventory.issued_tokens = inventory.issued_tokens.checked_add(quantity)
        .unwrap_or_else(|| panic_with_error!(env, IssueError::InventoryOverflow));
    
    storage::update_inventory(env, commodity_type, &inventory);
    storage::set_token_owner(env, &token_id, issuer);
    metadata::add_to_commodity_index(env, commodity_type, &token_id);

    env.events().publish(
        (symbol_short!("issued"), issuer.clone()),
        (token_id.clone(), commodity_type.clone(), quantity),
    );

    token_id
}


fn validate_issuer(env: &Env, issuer: &Address) {
    let admin = storage::get_admin(env);
    let authorized_issuers = storage::get_authorized_issuers(env);
    if *issuer != admin && !authorized_issuers.iter().any(|auth_issuer| auth_issuer == *issuer) {
        panic_with_error!(env, IssueError::UnauthorizedIssuer);
    }
}

// Generates a unique ID by hashing manually combined bytes of key inputs and a nonce.
fn generate_token_id(
    env: &Env,
    quantity: u32,
    expiration_date: u64,
    verification_data: &BytesN<32>,
    timestamp: u64,
    nonce: u64,
) -> BytesN<32> { // Return type BytesN

    let mut buffer = StdVec::new();

    // Append bytes for easily serializable components
    buffer.extend_from_slice(&quantity.to_be_bytes());
    buffer.extend_from_slice(&expiration_date.to_be_bytes());
    buffer.extend_from_slice(&verification_data.to_array());
    buffer.extend_from_slice(&timestamp.to_be_bytes());
    buffer.extend_from_slice(&nonce.to_be_bytes());

    // Create Soroban Bytes from the collected bytes
    let bytes_to_hash: Bytes = Bytes::from_slice(env, &buffer);

    let hash_result: soroban_sdk::crypto::Hash<32> = env.crypto().sha256(&bytes_to_hash);

    // Convert Hash into the required BytesN return type.
    hash_result.into()
}