#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

mod alerts;
mod datatypes;
mod error;
mod incentives;
mod utils;
mod water_usage;

#[cfg(test)]
mod test;

pub use datatypes::*;
pub use error::*;

#[contract]
pub struct WaterManagementContract;

#[contractimpl]
impl WaterManagementContract {
    /// Initialize the contract with an admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        Ok(())
    }
    
    /// Record water usage data for a parcel or crop
    pub fn record_usage(
        env: Env,
        usage_id: BytesN<32>,
        farmer_id: Address,
        parcel_id: BytesN<32>,
        volume: i128,
        data_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        farmer_id.require_auth();
        
        // Record the usage
        water_usage::record_usage(&env, usage_id.clone(), farmer_id, parcel_id, volume, data_hash)?;
        
        // Check for alerts
        let _ = alerts::check_usage_and_alert(&env, usage_id.clone());
        
        // Process automatic incentive
        let _ = incentives::process_automatic_incentive(&env, usage_id);
        
        Ok(())
    }
    
    /// Issue incentive reward for efficient water usage
    pub fn issue_incentive(
        env: Env,
        usage_id: BytesN<32>,
        base_reward: i128,
    ) -> Result<(), ContractError> {
        // Get the usage to verify farmer authorization
        let usage = water_usage::get_usage(&env, usage_id.clone())?;
        usage.farmer_id.require_auth();
        
        incentives::issue_incentive(&env, usage_id, base_reward)
    }
    
    /// Generate alert for excessive water consumption
    pub fn generate_alert(
        env: Env,
        alert_id: BytesN<32>,
        farmer_id: Address,
        parcel_id: BytesN<32>,
        alert_type: AlertType,
        message: String,
    ) -> Result<(), ContractError> {
        farmer_id.require_auth();
        
        alerts::generate_alert(&env, alert_id, farmer_id, parcel_id, alert_type, message)
    }
    
    /// Get water usage report for a farmer or parcel
    pub fn get_usage_report(
        env: Env,
        farmer_id: Address,
        parcel_id: Option<BytesN<32>>,
        period_start: u64,
        period_end: u64,
    ) -> Result<UsageReport, ContractError> {
        water_usage::get_usage_report(&env, farmer_id, parcel_id, period_start, period_end)
    }
    
    /// Set water usage threshold for a parcel (admin only)
    pub fn set_threshold(
        env: Env,
        admin: Address,
        parcel_id: BytesN<32>,
        daily_limit: i128,
        weekly_limit: i128,
        monthly_limit: i128,
    ) -> Result<(), ContractError> {
        incentives::set_threshold(&env, admin, parcel_id, daily_limit, weekly_limit, monthly_limit)
    }
    
    /// Get water usage threshold for a parcel
    pub fn get_threshold(env: Env, parcel_id: BytesN<32>) -> Result<WaterThreshold, ContractError> {
        incentives::get_threshold(&env, parcel_id)
    }
    
    /// Get water usage record by ID
    pub fn get_usage(env: Env, usage_id: BytesN<32>) -> Result<WaterUsage, ContractError> {
        water_usage::get_usage(&env, usage_id)
    }
    
    /// Get all usage records for a farmer
    pub fn get_farmer_usages(env: Env, farmer_id: Address) -> Vec<WaterUsage> {
        water_usage::get_farmer_usages(&env, farmer_id)
    }
    
    /// Get all usage records for a parcel
    pub fn get_parcel_usages(env: Env, parcel_id: BytesN<32>) -> Vec<WaterUsage> {
        water_usage::get_parcel_usages(&env, parcel_id)
    }
    
    /// Get incentive record by usage ID
    pub fn get_incentive(env: Env, usage_id: BytesN<32>) -> Result<Incentive, ContractError> {
        incentives::get_incentive(&env, usage_id)
    }
    
    /// Get all incentives for a farmer
    pub fn get_farmer_incentives(env: Env, farmer_id: Address) -> Vec<Incentive> {
        incentives::get_farmer_incentives(&env, farmer_id)
    }
    
    /// Calculate total rewards earned by a farmer in a time period
    pub fn calculate_farmer_rewards(
        env: Env,
        farmer_id: Address,
        period_start: u64,
        period_end: u64,
    ) -> Result<i128, ContractError> {
        incentives::calculate_farmer_rewards(&env, farmer_id, period_start, period_end)
    }
    
    /// Get alert by ID
    pub fn get_alert(env: Env, alert_id: BytesN<32>) -> Result<Alert, ContractError> {
        alerts::get_alert(&env, alert_id)
    }
    
    /// Resolve an alert (mark as resolved)
    pub fn resolve_alert(
        env: Env,
        alert_id: BytesN<32>,
        resolver: Address,
    ) -> Result<(), ContractError> {
        resolver.require_auth();
        alerts::resolve_alert(&env, alert_id, resolver)
    }
    
    /// Get all alerts for a farmer
    pub fn get_farmer_alerts(
        env: Env,
        farmer_id: Address,
        include_resolved: bool,
    ) -> Vec<Alert> {
        alerts::get_farmer_alerts(&env, farmer_id, include_resolved)
    }
}
