use crate::DataKey;
use soroban_sdk::{BytesN, Env};

/// Generate a unique ID for trade offers and barter agreements
pub fn generate_id(env: &Env) -> BytesN<32> {
    let counter: u32 = env
        .storage()
        .instance()
        .get(&DataKey::OfferCounter)
        .unwrap_or(0);

    let new_counter = counter + 1;
    env.storage()
        .instance()
        .set(&DataKey::OfferCounter, &new_counter);

    // Create a unique ID using timestamp, counter, and ledger sequence
    let timestamp = env.ledger().timestamp();
    let sequence = env.ledger().sequence();

    // Combine timestamp, counter, and sequence for uniqueness
    let mut id_bytes = [0u8; 32];
    id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
    id_bytes[8..12].copy_from_slice(&new_counter.to_be_bytes());
    id_bytes[12..16].copy_from_slice(&sequence.to_be_bytes());

    // Fill remaining bytes with a pattern to ensure uniqueness
    for i in 16..32 {
        id_bytes[i] = ((timestamp + new_counter as u64 + sequence as u64) % 256) as u8;
    }

    BytesN::from_array(env, &id_bytes)
}
