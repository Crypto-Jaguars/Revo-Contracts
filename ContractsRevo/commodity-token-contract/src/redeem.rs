use soroban_sdk::{Address, BytesN, Env};
use crate::{storage, metadata};

pub fn redeem_token(
    env: &Env,
    token_id: &BytesN<32>,
    redeemer: &Address,
    quantity: u32,
) {
    // Get token data
    let mut token = metadata::get_token_metadata(env, token_id);
    
    // Ensure token exists
    if token.quantity == 0 {
        panic!("Token does not exist or has been fully redeemed");
    }
    
    // Ensure redeemer owns the token
    let owner = storage::get_token_owner(env, token_id);
    if owner != *redeemer {
        panic!("Only the token owner can redeem");
    }
    
    // Ensure valid redemption quantity
    if quantity > token.quantity {
        panic!("Redemption quantity exceeds available token amount");
    }
    
    // Check if token has expired
    let current_time = env.ledger().timestamp();
    if current_time > token.expiration_date {
        panic!("Token has expired");
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
        .expect("Inventory underflow");

    inventory.total_quantity = inventory
        .total_quantity
        .checked_sub(quantity)
        .expect("Total inventory underflow");

    storage::update_inventory(env, &token.commodity_type, &inventory);
    
    // Emit redemption event
    env.events().publish(
        ("token_redeemed", redeemer),
        (token_id.clone(), token.commodity_type.clone(), quantity),
    );
    
    // Trigger physical redemption process via event
    env.events().publish(
        ("physical_redemption_initiated", redeemer),
        (token_id.clone(), token.storage_location.clone(), quantity),
    );
}