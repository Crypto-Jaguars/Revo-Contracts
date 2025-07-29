use crate::{datatypes::*, error::ContractError, utils, water_usage};
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

/// Issues incentive rewards for efficient water usage
pub fn issue_incentive(
    env: &Env,
    usage_id: BytesN<32>,
    base_reward: i128,
) -> Result<(), ContractError> {
    // Get the water usage record
    let usage = water_usage::get_usage(env, usage_id.clone())?;

    // Check if incentive already exists for this usage
    let incentive_key = DataKey::Incentive(usage_id.clone());
    if env.storage().persistent().has(&incentive_key) {
        return Err(ContractError::IncentiveAlreadyExists);
    }

    // Get threshold for the parcel
    let threshold = env
        .storage()
        .persistent()
        .get::<DataKey, WaterThreshold>(&DataKey::Threshold(usage.parcel_id.clone()))
        .ok_or(ContractError::ThresholdNotFound)?;

    // Check if usage qualifies for incentive
    if !utils::qualifies_for_incentive(usage.volume, threshold.daily_limit) {
        return Err(ContractError::InsufficientEfficiency);
    }

    // Calculate reward amount based on efficiency
    let reward_amount =
        utils::calculate_reward_amount(usage.volume, threshold.daily_limit, base_reward);

    if reward_amount <= 0 {
        return Err(ContractError::InvalidRewardAmount);
    }

    let timestamp = env.ledger().timestamp();

    // Create incentive record
    let incentive = Incentive {
        farmer_id: usage.farmer_id.clone(),
        reward_amount,
        timestamp,
        usage_id: usage_id.clone(),
    };

    // Store the incentive
    env.storage().persistent().set(&incentive_key, &incentive);

    // Update farmer's incentives list
    let farmer_incentives_key = DataKey::FarmerIncentives(usage.farmer_id.clone());
    let mut farmer_incentives: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&farmer_incentives_key)
        .unwrap_or_else(|| Vec::new(env));

    farmer_incentives.push_back(usage_id.clone());
    env.storage()
        .persistent()
        .set(&farmer_incentives_key, &farmer_incentives);

    // Emit incentive issued event
    env.events().publish(
        (
            Symbol::new(env, "incentive_issued"),
            usage.farmer_id.clone(),
        ),
        (usage_id.clone(), reward_amount, timestamp),
    );

    // Emit loyalty token reward event for integration
    env.events().publish(
        (
            Symbol::new(env, "loyalty_reward_earned"),
            usage.farmer_id.clone(),
        ),
        (usage.parcel_id.clone(), reward_amount),
    );

    Ok(())
}

/// Retrieves incentive record by usage ID
pub fn get_incentive(env: &Env, usage_id: BytesN<32>) -> Result<Incentive, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Incentive(usage_id))
        .ok_or(ContractError::IncentiveNotFound)
}

/// Gets all incentives for a farmer
pub fn get_farmer_incentives(env: &Env, farmer_id: Address) -> Vec<Incentive> {
    let incentive_usage_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::FarmerIncentives(farmer_id))
        .unwrap_or_else(|| Vec::new(env));

    let mut incentives = Vec::new(env);
    for usage_id in incentive_usage_ids.iter() {
        if let Some(incentive) = env
            .storage()
            .persistent()
            .get::<DataKey, Incentive>(&DataKey::Incentive(usage_id.clone()))
        {
            incentives.push_back(incentive);
        }
    }

    incentives
}

/// Calculates total rewards earned by a farmer in a time period
pub fn calculate_farmer_rewards(
    env: &Env,
    farmer_id: Address,
    period_start: u64,
    period_end: u64,
) -> Result<i128, ContractError> {
    if period_start >= period_end {
        return Err(ContractError::InvalidTimestamp);
    }

    let incentives = get_farmer_incentives(env, farmer_id);
    let mut total_rewards = 0i128;

    for incentive in incentives.iter() {
        if incentive.timestamp >= period_start && incentive.timestamp <= period_end {
            total_rewards += incentive.reward_amount;
        }
    }

    Ok(total_rewards)
}

/// Processes automatic incentive for a water usage record
pub fn process_automatic_incentive(env: &Env, usage_id: BytesN<32>) -> Result<(), ContractError> {
    // Default base reward amount (can be made configurable)
    const DEFAULT_BASE_REWARD: i128 = 100;

    // Try to issue incentive - will fail if not qualified or already exists
    match issue_incentive(env, usage_id.clone(), DEFAULT_BASE_REWARD) {
        Ok(()) => {
            // Emit automatic processing event
            env.events().publish(
                (Symbol::new(env, "automatic_incentive_processed"),),
                usage_id,
            );
            Ok(())
        }
        Err(ContractError::InsufficientEfficiency) => {
            // Not an error - just doesn't qualify
            Ok(())
        }
        Err(ContractError::IncentiveAlreadyExists) => {
            // Not an error - already processed
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Sets water usage threshold for a parcel (admin only)
pub fn set_threshold(
    env: &Env,
    admin: Address,
    parcel_id: BytesN<32>,
    daily_limit: i128,
    weekly_limit: i128,
    monthly_limit: i128,
) -> Result<(), ContractError> {
    // Require admin authorization
    utils::require_admin_auth(env, &admin)?;

    // Validate inputs
    utils::validate_identifier(env, &parcel_id)?;

    if daily_limit <= 0 || weekly_limit <= 0 || monthly_limit <= 0 {
        return Err(ContractError::InvalidThreshold);
    }

    // Ensure logical consistency (weekly >= daily * 7, monthly >= weekly * 4)
    if weekly_limit < daily_limit * 7 || monthly_limit < weekly_limit * 4 {
        return Err(ContractError::InvalidThreshold);
    }

    let threshold = WaterThreshold {
        parcel_id: parcel_id.clone(),
        daily_limit,
        weekly_limit,
        monthly_limit,
    };

    // Store the threshold
    env.storage()
        .persistent()
        .set(&DataKey::Threshold(parcel_id.clone()), &threshold);

    // Emit threshold set event
    env.events().publish(
        (Symbol::new(env, "threshold_set"), admin),
        (parcel_id, daily_limit, weekly_limit, monthly_limit),
    );

    Ok(())
}

/// Gets water usage threshold for a parcel
pub fn get_threshold(env: &Env, parcel_id: BytesN<32>) -> Result<WaterThreshold, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Threshold(parcel_id))
        .ok_or(ContractError::ThresholdNotFound)
}
