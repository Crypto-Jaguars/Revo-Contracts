use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Symbol};

const ADMIN: Symbol = symbol_short!("ADMIN");

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&ADMIN)
}

pub fn is_admin(env: &Env, address: &Address) -> bool {
    if let Some(admin) = get_admin(env) {
        admin == *address
    } else {
        false
    }
}

pub fn generate_id(env: &Env, counter: u64) -> BytesN<32> {
    let timestamp = env.ledger().timestamp(); // u64
    let sequence = env.ledger().sequence(); // u32 - this was the issue!

    // Create a unique identifier by hashing timestamp, sequence, and counter
    let mut data = [0u8; 20]; // Reduced size to fit actual data
    data[0..8].copy_from_slice(&timestamp.to_be_bytes());
    data[8..12].copy_from_slice(&sequence.to_be_bytes()); // u32 = 4 bytes
    data[12..20].copy_from_slice(&counter.to_be_bytes());

    // Convert to Bytes first, then hash
    let bytes = Bytes::from_array(env, &data);
    env.crypto().sha256(&bytes).into()
}

pub fn validate_address(_env: &Env, address: &Address) -> bool {
    !address.to_string().is_empty()
}

pub fn validate_positive_amount(amount: i128) -> bool {
    amount > 0
}

pub fn calculate_late_fee(base_amount: i128, days_late: u64) -> i128 {
    let daily_rate = base_amount / 100; // 1% per day
    let late_fee = daily_rate * days_late as i128;
    let max_fee = base_amount / 5; // 20% cap

    if late_fee > max_fee {
        max_fee
    } else {
        late_fee
    }
}
