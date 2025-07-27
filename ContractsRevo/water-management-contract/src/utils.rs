use soroban_sdk::{Address, BytesN, Env, Vec};
use crate::{datatypes::*, error::ContractError};

/// Validates that a water volume is within acceptable limits
pub fn validate_water_volume(volume: i128) -> Result<(), ContractError> {
    if volume <= 0 {
        return Err(ContractError::InvalidVolume);
    }
    
    // Maximum daily usage per parcel: 100,000 liters (reasonable for large agricultural parcels)
    const MAX_DAILY_VOLUME: i128 = 100_000;
    if volume > MAX_DAILY_VOLUME {
        return Err(ContractError::InvalidVolume);
    }
    
    Ok(())
}

/// Validates timestamp is not in the future and not too old
pub fn validate_timestamp(env: &Env, timestamp: u64) -> Result<(), ContractError> {
    let current_time = env.ledger().timestamp();
    
    // Don't allow future timestamps
    if timestamp > current_time {
        return Err(ContractError::InvalidTimestamp);
    }
    
    // Don't allow timestamps older than 30 days (2,592,000 seconds)
    const MAX_AGE: u64 = 2_592_000;
    if current_time - timestamp > MAX_AGE {
        return Err(ContractError::InvalidTimestamp);
    }
    
    Ok(())
}

/// Validates that identifiers are not empty (all zeros)
pub fn validate_identifier(env: &Env, id: &BytesN<32>) -> Result<(), ContractError> {
    let empty_id = BytesN::from_array(env, &[0u8; 32]);
    if *id == empty_id {
        return Err(ContractError::InvalidInput);
    }
    Ok(())
}

/// Validates data hash is not empty
pub fn validate_data_hash(env: &Env, hash: &BytesN<32>) -> Result<(), ContractError> {
    let empty_hash = BytesN::from_array(env, &[0u8; 32]);
    if *hash == empty_hash {
        return Err(ContractError::InvalidDataHash);
    }
    Ok(())
}

/// Calculates efficiency score based on usage vs threshold
pub fn calculate_efficiency_score(usage: i128, threshold: i128) -> u32 {
    if threshold <= 0 {
        return 0;
    }
    
    let efficiency_ratio = (threshold as f64) / (usage as f64);
    let score = (efficiency_ratio * 100.0).min(100.0).max(0.0) as u32;
    
    // Bonus points for being significantly under threshold
    if usage <= threshold / 2 {
        (score + 10).min(100)
    } else {
        score
    }
}

/// Determines if usage qualifies for incentive rewards
pub fn qualifies_for_incentive(usage: i128, threshold: i128) -> bool {
    // Qualify if usage is 80% or less of the threshold
    usage <= (threshold * 80) / 100
}

/// Calculates reward amount based on efficiency
pub fn calculate_reward_amount(usage: i128, threshold: i128, base_reward: i128) -> i128 {
    if !qualifies_for_incentive(usage, threshold) {
        return 0;
    }
    
    let efficiency_score = calculate_efficiency_score(usage, threshold);
    
    // Scale reward based on efficiency score
    match efficiency_score {
        90..=100 => base_reward * 2,      // Excellent efficiency
        80..=89 => (base_reward * 15) / 10, // Good efficiency  
        70..=79 => base_reward,           // Acceptable efficiency
        _ => base_reward / 2,             // Minimal efficiency
    }
}

/// Checks if admin authorization is required and validates it
pub fn require_admin_auth(env: &Env, caller: &Address) -> Result<(), ContractError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::NotInitialized)?;
    
    if *caller != admin {
        return Err(ContractError::Unauthorized);
    }
    
    caller.require_auth();
    Ok(())
}

/// Gets the current day timestamp (start of day)
pub fn get_day_start(timestamp: u64) -> u64 {
    const SECONDS_PER_DAY: u64 = 86400;
    (timestamp / SECONDS_PER_DAY) * SECONDS_PER_DAY
}

/// Gets the current week timestamp (start of week)
pub fn get_week_start(timestamp: u64) -> u64 {
    const SECONDS_PER_WEEK: u64 = 604800;
    (timestamp / SECONDS_PER_WEEK) * SECONDS_PER_WEEK
}

/// Gets the current month timestamp (approximate start of month)
pub fn get_month_start(timestamp: u64) -> u64 {
    const SECONDS_PER_MONTH: u64 = 2592000; // 30 days
    (timestamp / SECONDS_PER_MONTH) * SECONDS_PER_MONTH
}
