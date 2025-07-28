use crate::{datatypes::*, error::ContractError, incentives, utils, water_usage};
use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

/// Generates alert for excessive water consumption
pub fn generate_alert(
    env: &Env,
    alert_id: BytesN<32>,
    farmer_id: Address,
    parcel_id: BytesN<32>,
    alert_type: AlertType,
    message: String,
) -> Result<(), ContractError> {
    // Validate inputs
    utils::validate_identifier(env, &alert_id)?;
    utils::validate_identifier(env, &parcel_id)?;

    if message.is_empty() {
        return Err(ContractError::InvalidInput);
    }

    // Check if alert already exists
    if env
        .storage()
        .persistent()
        .has(&DataKey::Alert(alert_id.clone()))
    {
        return Err(ContractError::AlertAlreadyExists);
    }

    let timestamp = env.ledger().timestamp();

    // Create alert record
    let alert = Alert {
        alert_id: alert_id.clone(),
        farmer_id: farmer_id.clone(),
        parcel_id: parcel_id.clone(),
        alert_type: alert_type.clone(),
        message: message.clone(),
        timestamp,
        resolved: false,
    };

    // Store the alert
    env.storage()
        .persistent()
        .set(&DataKey::Alert(alert_id.clone()), &alert);

    // Update farmer alerts index
    let farmer_alerts_key = DataKey::FarmerAlerts(farmer_id.clone());
    let mut farmer_alerts: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&farmer_alerts_key)
        .unwrap_or_else(|| Vec::new(env));

    farmer_alerts.push_back(alert_id.clone());
    env.storage()
        .persistent()
        .set(&farmer_alerts_key, &farmer_alerts);

    // Emit alert generated event
    env.events().publish(
        (Symbol::new(env, "alert_generated"), farmer_id.clone()),
        (alert_id.clone(), parcel_id.clone(), message),
    );

    Ok(())
}

/// Checks water usage against thresholds and generates alerts if needed
pub fn check_usage_and_alert(env: &Env, usage_id: BytesN<32>) -> Result<(), ContractError> {
    // Get the water usage record
    let usage = water_usage::get_usage(env, usage_id.clone())?;

    // Get threshold for the parcel
    let threshold_result = incentives::get_threshold(env, usage.parcel_id.clone());
    if threshold_result.is_err() {
        // No threshold set - cannot check for alerts
        return Ok(());
    }

    let threshold = threshold_result.unwrap();
    let _current_time = env.ledger().timestamp();

    // Check daily usage
    let day_start = utils::get_day_start(usage.timestamp);
    let day_end = day_start + 86400; // 24 hours

    let daily_report = water_usage::get_usage_report(
        env,
        usage.farmer_id.clone(),
        Some(usage.parcel_id.clone()),
        day_start,
        day_end,
    )?;

    if daily_report.total_usage > threshold.daily_limit {
        let alert_id = generate_alert_id(env, &usage.farmer_id, &usage.parcel_id, "daily_exceeded");
        let message = String::from_str(env, "Daily water limit exceeded");

        match generate_alert(
            env,
            alert_id,
            usage.farmer_id.clone(),
            usage.parcel_id.clone(),
            AlertType::ThresholdExceeded,
            message,
        ) {
            Ok(()) => {}
            Err(ContractError::AlertAlreadyExists) => {
                // Expected - alert already exists for this period
            }
            Err(e) => return Err(e),
        }
    }

    // Check weekly usage
    let week_start = utils::get_week_start(usage.timestamp);
    let week_end = week_start + 604800; // 7 days

    let weekly_report = water_usage::get_usage_report(
        env,
        usage.farmer_id.clone(),
        Some(usage.parcel_id.clone()),
        week_start,
        week_end,
    )?;

    if weekly_report.total_usage > threshold.weekly_limit {
        let alert_id =
            generate_alert_id(env, &usage.farmer_id, &usage.parcel_id, "weekly_exceeded");
        let message = String::from_str(env, "Weekly water limit exceeded");

        match generate_alert(
            env,
            alert_id,
            usage.farmer_id.clone(),
            usage.parcel_id.clone(),
            AlertType::ThresholdExceeded,
            message,
        ) {
            Ok(()) => {}
            Err(ContractError::AlertAlreadyExists) => {
                // Expected - alert already exists for this period
            }
            Err(e) => return Err(e),
        }
    }

    // Check for excessive single usage (more than 50% of daily limit in one record)
    if usage.volume > threshold.daily_limit / 2 {
        let alert_id =
            generate_alert_id(env, &usage.farmer_id, &usage.parcel_id, "excessive_single");
        let message = String::from_str(env, "Excessive single usage detected");

        match generate_alert(
            env,
            alert_id,
            usage.farmer_id.clone(),
            usage.parcel_id.clone(),
            AlertType::ExcessiveUsage,
            message,
        ) {
            Ok(()) => {}
            Err(ContractError::AlertAlreadyExists) => {
                // Expected - alert already exists for this period
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Resolves an alert (marks it as resolved)
pub fn resolve_alert(
    env: &Env,
    alert_id: BytesN<32>,
    resolver: Address,
) -> Result<(), ContractError> {
    // Get the alert
    let mut alert: Alert = env
        .storage()
        .persistent()
        .get(&DataKey::Alert(alert_id.clone()))
        .ok_or(ContractError::AlertNotFound)?;

    // Check if already resolved
    if alert.resolved {
        return Ok(());
    }

    // Mark as resolved
    alert.resolved = true;
    env.storage()
        .persistent()
        .set(&DataKey::Alert(alert_id.clone()), &alert);

    // Emit alert resolved event
    env.events().publish(
        (Symbol::new(env, "alert_resolved"), resolver),
        (alert_id, alert.farmer_id.clone()),
    );

    Ok(())
}

/// Gets alert by ID
pub fn get_alert(env: &Env, alert_id: BytesN<32>) -> Result<Alert, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Alert(alert_id))
        .ok_or(ContractError::AlertNotFound)
}

/// Gets all alerts for a farmer
pub fn get_farmer_alerts(env: &Env, farmer_id: Address, include_resolved: bool) -> Vec<Alert> {
    let farmer_alerts_key = DataKey::FarmerAlerts(farmer_id.clone());
    let alert_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&farmer_alerts_key)
        .unwrap_or_else(|| Vec::new(env));

    let mut result = Vec::new(env);

    for alert_id in alert_ids.iter() {
        if let Ok(alert) = get_alert(env, alert_id.clone()) {
            // Filter based on resolved status
            if include_resolved || !alert.resolved {
                result.push_back(alert);
            }
        }
    }

    result
}

/// Generates a deterministic alert ID based on farmer, parcel, and alert type
fn generate_alert_id(
    env: &Env,
    _farmer_id: &Address,
    parcel_id: &BytesN<32>,
    alert_suffix: &str,
) -> BytesN<32> {
    // Create a simple deterministic ID by combining inputs
    // Use a simplified approach that works with Soroban SDK
    let mut id_bytes = [0u8; 32];

    // Use timestamp and inputs to create unique ID
    let timestamp = env.ledger().timestamp();
    let timestamp_bytes = timestamp.to_be_bytes();

    // Copy timestamp bytes
    id_bytes[0..8].copy_from_slice(&timestamp_bytes);

    // Add some bytes from parcel_id (which is already BytesN<32>)
    let parcel_bytes = parcel_id.to_array();
    id_bytes[8..(16 + 8)].copy_from_slice(&parcel_bytes[..16]);

    // Add suffix influence
    let suffix_bytes = alert_suffix.as_bytes();
    for i in 0..8 {
        if i < suffix_bytes.len() {
            id_bytes[24 + i] = suffix_bytes[i];
        }
    }

    BytesN::from_array(env, &id_bytes)
}
