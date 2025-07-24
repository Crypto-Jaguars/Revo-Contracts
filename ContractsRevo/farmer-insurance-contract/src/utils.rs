use soroban_sdk::{Env, Bytes, BytesN, contracttype};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Policy(BytesN<32>),
    Claim(BytesN<32>),
    PolicyCount,
    ClaimCount,
}

pub fn generate_policy_id(env: &Env) -> BytesN<32> {
    let count: u64 = env
        .storage()
        .instance()
        .get::<_, u64>(&DataKey::PolicyCount)
        .unwrap_or(0);

    let new_count = count + 1;
    env.storage().instance().set(&DataKey::PolicyCount, &new_count);

    let timestamp = env.ledger().timestamp();
    let mut buffer = Bytes::new(env);
    buffer.append(&Bytes::from_slice(env, &timestamp.to_be_bytes()));
    buffer.append(&Bytes::from_slice(env, &new_count.to_be_bytes()));
    let hash = env.crypto().sha256(&buffer);
    hash.to_bytes()
}

pub fn generate_claim_id(env: &Env) -> BytesN<32> {
    let count: u64 = env
        .storage()
        .instance()
        .get::<_, u64>(&DataKey::ClaimCount)
        .unwrap_or(0);

    let new_count = count + 1;
    env.storage().instance().set(&DataKey::ClaimCount, &new_count);

    let timestamp = env.ledger().timestamp();
    let mut buffer = Bytes::new(env);
    buffer.append(&Bytes::from_slice(env, &timestamp.to_be_bytes()));
    buffer.append(&Bytes::from_slice(env, &new_count.to_be_bytes()));
    let hash = env.crypto().sha256(&buffer);
    hash.to_bytes()
}
