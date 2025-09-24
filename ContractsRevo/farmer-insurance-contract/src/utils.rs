use soroban_sdk::{contracterror, contracttype, Bytes, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Policy(BytesN<32>),
    Claim(BytesN<32>),
    PolicyCount,
    ClaimCount,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    PolicyCountOverflow = 1,
    ClaimCountOverflow = 2,
}

pub fn generate_policy_id(env: &Env) -> Result<BytesN<32>, ContractError> {
    let count: u64 = env
        .storage()
        .persistent()
        .get::<_, u64>(&DataKey::PolicyCount)
        .unwrap_or(0);

    let new_count = count
        .checked_add(1)
        .ok_or(ContractError::PolicyCountOverflow)?;

    env.storage()
        .persistent()
        .set(&DataKey::PolicyCount, &new_count);
    // Set TTL for persistent storage (~30 days)
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::PolicyCount, 17280, 17280);

    let timestamp = env.ledger().timestamp();
    let mut buffer = Bytes::new(env);
    buffer.append(&Bytes::from_slice(env, &timestamp.to_be_bytes()));
    buffer.append(&Bytes::from_slice(env, &new_count.to_be_bytes()));
    let hash = env.crypto().sha256(&buffer);
    Ok(hash.to_bytes())
}

pub fn generate_claim_id(env: &Env) -> Result<BytesN<32>, ContractError> {
    let count: u64 = env
        .storage()
        .persistent()
        .get::<_, u64>(&DataKey::ClaimCount)
        .unwrap_or(0);

    let new_count = count
        .checked_add(1)
        .ok_or(ContractError::ClaimCountOverflow)?;

    env.storage()
        .persistent()
        .set(&DataKey::ClaimCount, &new_count);
    // Set TTL for persistent storage (~30 days)
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::ClaimCount, 17280, 17280);

    let timestamp = env.ledger().timestamp();
    let mut buffer = Bytes::new(env);
    buffer.append(&Bytes::from_slice(env, &timestamp.to_be_bytes()));
    buffer.append(&Bytes::from_slice(env, &new_count.to_be_bytes()));
    let hash = env.crypto().sha256(&buffer);
    Ok(hash.to_bytes())
}
