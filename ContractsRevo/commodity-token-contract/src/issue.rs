use soroban_sdk::{contracterror, Address, BytesN, Env, String, Symbol};

use crate::storage::DataKey;
use crate::{metadata, storage, validate, CommodityBackedToken, ContractError};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
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

// Implementation for converting ContractError to IssueError
impl From<ContractError> for IssueError {
    fn from(err: ContractError) -> Self {
        match err {
            ContractError::Unauthorized => IssueError::UnauthorizedIssuer,
            _ => IssueError::IdGenerationError,
        }
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
) -> Result<BytesN<32>, IssueError> {
    validate_issuer(env, issuer)?;

    if !validate::validate_commodity(env, commodity_type, verification_data) {
        return Err(IssueError::InvalidCommodityData);
    }

    let current_time = env.ledger().timestamp();
    if expiration_date <= current_time {
        return Err(IssueError::InvalidExpirationDate);
    }

    let mut inventory = storage::get_inventory(env, commodity_type);
    if inventory.available_quantity < quantity {
        return Err(IssueError::InsufficientInventory);
    }

    let nonce_key = DataKey::TokenNonce;
    let current_nonce: u64 = env.storage().instance().get(&nonce_key).unwrap_or(0u64);
    let next_nonce = current_nonce
        .checked_add(1)
        .ok_or(IssueError::NonceOverflow)?;
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
        current_nonce,
    )?;

    storage::store_token(env, &token_id, &token);

    inventory.available_quantity = inventory
        .available_quantity
        .checked_sub(quantity)
        .ok_or(IssueError::InventoryUnderflow)?;
    inventory.issued_tokens = inventory
        .issued_tokens
        .checked_add(quantity)
        .ok_or(IssueError::InventoryOverflow)?;

    storage::update_inventory(env, commodity_type, &inventory).map_err(|err| match err {
        ContractError::Unauthorized => IssueError::UnauthorizedIssuer,
        _ => IssueError::InventoryUnderflow,
    })?;

    storage::set_token_owner(env, &token_id, issuer);
    metadata::add_to_commodity_index(env, commodity_type, &token_id);

    env.events().publish(
        (Symbol::new(env, "issued"), issuer.clone()),
        (token_id.clone(), commodity_type.clone(), quantity),
    );

    Ok(token_id)
}

fn validate_issuer(env: &Env, issuer: &Address) -> Result<(), IssueError> {
    let admin = storage::get_admin(env);
    let authorized_issuers = storage::get_authorized_issuers(env);
    if *issuer != admin
        && !authorized_issuers
            .iter()
            .any(|auth_issuer| auth_issuer == *issuer)
    {
        return Err(IssueError::UnauthorizedIssuer);
    }
    Ok(())
}

// Generates a unique ID by hashing manually combined bytes of key inputs and a nonce.
fn generate_token_id(
    env: &Env,
    quantity: u32,
    expiration_date: u64,
    verification_data: &BytesN<32>,
    timestamp: u64,
    nonce: u64,
) -> Result<BytesN<32>, IssueError> {
    let mut buffer = [0u8; 60]; // 4 + 8 + 32 + 8 + 8 bytes
    let mut offset = 0;

    // Copy all data into fixed buffer
    buffer[offset..offset + 4].copy_from_slice(&quantity.to_be_bytes());
    offset += 4;

    buffer[offset..offset + 8].copy_from_slice(&expiration_date.to_be_bytes());
    offset += 8;

    // Get the raw bytes from BytesN<32>
    let verification_bytes = verification_data.to_array();
    buffer[offset..offset + 32].copy_from_slice(&verification_bytes);
    offset += 32;

    buffer[offset..offset + 8].copy_from_slice(&timestamp.to_be_bytes());
    offset += 8;

    buffer[offset..offset + 8].copy_from_slice(&nonce.to_be_bytes());

    // Create a Soroban Bytes object from fixed buffer
    let bytes = soroban_sdk::Bytes::from_slice(env, &buffer);

    // Hash the bytes
    let hash_result = env.crypto().sha256(&bytes);

    Ok(hash_result.into())
}
