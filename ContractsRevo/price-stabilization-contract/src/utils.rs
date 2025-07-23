use crate::datatype::{DataKey, FarmerCrop, PriceData, StabilizationError, StabilizationFund};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

/// Utility functions for price calculations and oracle interactions
pub struct PriceUtils;

impl PriceUtils {
    /// Calculate the average price from multiple oracles
    pub fn calculate_average_price(env: &Env, crop_type: &String) -> Result<i128, StabilizationError> {
        
        // Check if price data exists for this crop type
        let price_key = DataKey::MarketPrice(crop_type.clone());
        if !env.storage().persistent().has(&price_key) {
            return Err(StabilizationError::PriceDataNotAvailable);
        }
        
        // Get the price data
        let price_data: PriceData = env.storage().persistent().get(&price_key).unwrap();
        
        Ok(price_data.price)
    }
    
    /// Calculate the price difference between threshold and current price
    pub fn calculate_price_difference(
        env: &Env,
        fund_id: &BytesN<32>,
    ) -> Result<i128, StabilizationError> {
        // Check if fund exists
        let fund_key = DataKey::Fund(fund_id.clone());
        if !env.storage().persistent().has(&fund_key) {
            return Err(StabilizationError::FundNotFound);
        }
        
        // Get the fund
        let fund: StabilizationFund = env.storage().persistent().get(&fund_key).unwrap();
        
        // Get the current market price
        let current_price = Self::calculate_average_price(env, &fund.crop_type)?;
        
        // Calculate the difference (will be positive if threshold is higher than market price)
        let difference = fund.price_threshold - current_price;
        
        if difference <= 0 {
            // Price is at or above threshold, no payout needed
            return Ok(0);
        }
        
        Ok(difference)
    }
    
    /// Check if a fund has sufficient balance for potential payouts
    pub fn check_fund_sufficiency(
        env: &Env,
        fund_id: &BytesN<32>,
        farmers: &Vec<Address>,
    ) -> Result<bool, StabilizationError> {
        // Check if fund exists
        let fund_key = DataKey::Fund(fund_id.clone());
        if !env.storage().persistent().has(&fund_key) {
            return Err(StabilizationError::FundNotFound);
        }
        
        // Get the fund
        let fund: StabilizationFund = env.storage().persistent().get(&fund_key).unwrap();
        
        // Calculate price difference
        let price_difference = Self::calculate_price_difference(env, fund_id)?;
        
        if price_difference <= 0 {
            // No payout needed
            return Ok(true);
        }
        
        // Calculate total production capacity of all farmers
        let mut total_capacity: i128 = 0;
        
        for farmer_address in farmers.iter() {
            // Check if farmer grows this crop type
            let farmer_crop_key = DataKey::FarmerCrops(farmer_address.clone(), fund.crop_type.clone());
            if env.storage().persistent().has(&farmer_crop_key) {
                let farmer_crop = env.storage().persistent().get::<DataKey, FarmerCrop>(&farmer_crop_key);
                total_capacity += farmer_crop.unwrap().production_capacity;
            }
        }
        
        // Calculate total payout needed
        let total_payout_needed = price_difference * total_capacity;
        
        // Check if fund has sufficient balance
        Ok(fund.total_balance >= total_payout_needed)
    }
    
    /// Validate timestamp is recent enough (within 1 hour)
    pub fn validate_timestamp(env: &Env, timestamp: u64) -> bool {
        let current_time = env.ledger().timestamp();
        let one_hour = 3600; // seconds
        
        // Check if timestamp is within the last hour
        current_time - timestamp <= one_hour
    }
}