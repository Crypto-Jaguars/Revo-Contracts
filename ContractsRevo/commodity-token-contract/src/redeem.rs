use soroban_sdk::{Address, BytesN, Env, Symbol};
use crate::{storage, metadata, RedeemError};

pub fn redeem_token(
    env: &Env,
    token_id: &BytesN<32>,
    redeemer: &Address,
    quantity: u32,
) -> Result<(), RedeemError> {
    // Get token data
    let token_result = metadata::get_token_metadata(env, token_id);
    
    if token_result.is_err() {
        return Err(RedeemError::TokenNotFound);
    }
    
    let mut token = token_result.unwrap();
    
    // Ensure token exists
    if token.quantity == 0 {
        return Err(RedeemError::InsufficientQuantity);
    }
    
    // Ensure redeemer owns the token
    let owner_result = storage::get_token_owner(env, token_id);
    if owner_result.is_err() || owner_result.unwrap() != *redeemer {
        return Err(RedeemError::NotTokenOwner);
    }
    
    // Ensure valid redemption quantity
    if quantity > token.quantity {
        return Err(RedeemError::InsufficientQuantity);
    }
    
    // Check if token has expired
    let current_time = env.ledger().timestamp();
    if current_time > token.expiration_date {
        return Err(RedeemError::TokenExpired);
    }
    
    // Update token data
    token.quantity -= quantity;
    
    // If fully redeemed, remove token
    if token.quantity == 0 {
        storage::remove_token(env, token_id);
        metadata::remove_from_commodity_index(env, &token.commodity_type, token_id);
    } else {
        storage::store_token(env, token_id, &token);
    }
    
    // Update inventory
    let mut inventory = storage::get_inventory(env, &token.commodity_type);

    inventory.issued_tokens = inventory.issued_tokens
        .checked_sub(quantity)
        .ok_or(RedeemError::InventoryUnderflow)?;

    inventory.total_quantity = inventory
        .total_quantity
        .checked_sub(quantity)
        .ok_or(RedeemError::InventoryUnderflow)?;

    storage::update_inventory(env, &token.commodity_type, &inventory).map_err(|_| RedeemError::InventoryUnderflow)?;
    
    // Emit redemption event
    env.events().publish(
        (Symbol::new(env, "token_redeemed"), redeemer.clone()),
        (token_id.clone(), token.commodity_type.clone(), quantity),
    );
    
    // Trigger physical redemption process via event
    env.events().publish(
        (Symbol::new(env, "physical_redemption_initiated"), redeemer.clone()),
        (token_id.clone(), token.storage_location.clone(), quantity),
    );
    
    Ok(())
}