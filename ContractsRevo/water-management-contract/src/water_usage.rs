use crate::{datatypes::*, error::ContractError, utils};
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

/// Records water usage data for a parcel or crop
pub fn record_usage(
    env: &Env,
    usage_id: BytesN<32>,
    farmer_id: Address,
    parcel_id: BytesN<32>,
    volume: i128,
    data_hash: BytesN<32>,
) -> Result<(), ContractError> {
    // Validate inputs
    utils::validate_identifier(env, &usage_id)?;
    utils::validate_identifier(env, &parcel_id)?;
    utils::validate_water_volume(volume)?;
    utils::validate_data_hash(env, &data_hash)?;

    // Check if usage record already exists
    if env
        .storage()
        .persistent()
        .has(&DataKey::Usage(usage_id.clone()))
    {
        return Err(ContractError::UsageAlreadyExists);
    }

    let timestamp = env.ledger().timestamp();
    utils::validate_timestamp(env, timestamp)?;

    // Create water usage record
    let usage = WaterUsage {
        usage_id: usage_id.clone(),
        farmer_id: farmer_id.clone(),
        parcel_id: parcel_id.clone(),
        volume,
        timestamp,
        data_hash,
    };

    // Store the usage record
    env.storage()
        .persistent()
        .set(&DataKey::Usage(usage_id.clone()), &usage);

    // Update farmer's usage list
    let farmer_usages_key = DataKey::FarmerUsages(farmer_id.clone());
    let mut farmer_usages: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&farmer_usages_key)
        .unwrap_or_else(|| Vec::new(env));

    farmer_usages.push_back(usage_id.clone());
    env.storage()
        .persistent()
        .set(&farmer_usages_key, &farmer_usages);

    // Update parcel's usage list
    let parcel_usages_key = DataKey::ParcelUsages(parcel_id.clone());
    let mut parcel_usages: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&parcel_usages_key)
        .unwrap_or_else(|| Vec::new(env));

    parcel_usages.push_back(usage_id.clone());
    env.storage()
        .persistent()
        .set(&parcel_usages_key, &parcel_usages);

    // Emit usage recorded event
    env.events().publish(
        (Symbol::new(env, "water_usage_recorded"), farmer_id.clone()),
        (usage_id.clone(), parcel_id.clone(), volume, timestamp),
    );

    Ok(())
}

/// Retrieves water usage record by ID
pub fn get_usage(env: &Env, usage_id: BytesN<32>) -> Result<WaterUsage, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Usage(usage_id))
        .ok_or(ContractError::UsageNotFound)
}

/// Generates usage report for a farmer or specific parcel
pub fn get_usage_report(
    env: &Env,
    farmer_id: Address,
    parcel_id: Option<BytesN<32>>,
    period_start: u64,
    period_end: u64,
) -> Result<UsageReport, ContractError> {
    if period_start >= period_end {
        return Err(ContractError::InvalidTimestamp);
    }

    let mut total_usage = 0i128;
    let mut usage_count = 0u32;
    let mut total_efficiency = 0u32;

    // Get usage records to analyze
    let usage_ids = if let Some(parcel) = parcel_id.clone() {
        // Get usage for specific parcel
        env.storage()
            .persistent()
            .get(&DataKey::ParcelUsages(parcel))
            .unwrap_or_else(|| Vec::<BytesN<32>>::new(env))
    } else {
        // Get all usage for farmer
        env.storage()
            .persistent()
            .get(&DataKey::FarmerUsages(farmer_id.clone()))
            .unwrap_or_else(|| Vec::<BytesN<32>>::new(env))
    };

    // Process each usage record
    for usage_id in usage_ids.iter() {
        if let Some(usage) = env
            .storage()
            .persistent()
            .get::<DataKey, WaterUsage>(&DataKey::Usage(usage_id.clone()))
        {
            // Check if usage falls within the specified period
            if usage.timestamp >= period_start && usage.timestamp <= period_end {
                // If parcel_id is specified, ensure it matches
                if parcel_id.is_none() || parcel_id.as_ref() == Some(&usage.parcel_id) {
                    total_usage += usage.volume;
                    usage_count += 1;

                    // Calculate efficiency if threshold exists
                    if let Some(threshold) =
                        env.storage().persistent().get::<DataKey, WaterThreshold>(
                            &DataKey::Threshold(usage.parcel_id.clone()),
                        )
                    {
                        let daily_efficiency =
                            utils::calculate_efficiency_score(usage.volume, threshold.daily_limit);
                        total_efficiency += daily_efficiency;
                    }
                }
            }
        }
    }

    // Calculate average efficiency score
    let efficiency_score = if usage_count > 0 {
        total_efficiency / usage_count
    } else {
        0
    };

    // Create a zero-filled BytesN for farmer-wide reports
    let is_farmer_wide = parcel_id.is_none();
    let report_parcel_id = parcel_id.unwrap_or_else(|| BytesN::from_array(env, &[0u8; 32]));

    Ok(UsageReport {
        farmer_id,
        parcel_id: report_parcel_id,
        is_farmer_wide,
        total_usage,
        period_start,
        period_end,
        efficiency_score,
    })
}

/// Gets all usage records for a farmer
pub fn get_farmer_usages(env: &Env, farmer_id: Address) -> Vec<WaterUsage> {
    let usage_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::FarmerUsages(farmer_id))
        .unwrap_or_else(|| Vec::new(env));

    let mut usages = Vec::new(env);
    for usage_id in usage_ids.iter() {
        if let Some(usage) = env
            .storage()
            .persistent()
            .get::<DataKey, WaterUsage>(&DataKey::Usage(usage_id.clone()))
        {
            usages.push_back(usage);
        }
    }

    usages
}

/// Gets all usage records for a parcel
pub fn get_parcel_usages(env: &Env, parcel_id: BytesN<32>) -> Vec<WaterUsage> {
    let usage_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::ParcelUsages(parcel_id))
        .unwrap_or_else(|| Vec::new(env));

    let mut usages = Vec::new(env);
    for usage_id in usage_ids.iter() {
        if let Some(usage) = env
            .storage()
            .persistent()
            .get::<DataKey, WaterUsage>(&DataKey::Usage(usage_id.clone()))
        {
            usages.push_back(usage);
        }
    }

    usages
}
