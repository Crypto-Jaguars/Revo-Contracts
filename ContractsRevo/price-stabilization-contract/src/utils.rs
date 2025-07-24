use crate::datatype::{DataKey, FarmerCrop, PriceData, StabilizationError, StabilizationFund, ChainlinkPriceFeed, ChainlinkPriceData};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

/// Utility functions for price calculations and oracle interactions
pub struct PriceUtils;

impl PriceUtils {
    /// Retrieve the stored market price for a crop type
    pub fn get_market_price(env: &Env, crop_type: &String) -> Result<i128, StabilizationError> {
        
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
        let current_price = Self::get_market_price(env, &fund.crop_type)?;
        
        // Calculate the difference with overflow protection
        let difference = fund.price_threshold
            .checked_sub(current_price)
            .ok_or(StabilizationError::InvalidInput)?;
        
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
                total_capacity = total_capacity
                    .checked_add(farmer_crop.unwrap().production_capacity)
                    .ok_or(StabilizationError::InvalidInput)?;
            }
        }
        
        // Calculate total payout needed with overflow check
        let total_payout_needed = price_difference
            .checked_mul(total_capacity)
            .ok_or(StabilizationError::InvalidInput)?;
        
        // Check if fund has sufficient balance
        Ok(fund.total_balance >= total_payout_needed)
    }
    
    /// Validate timestamp is recent enough (within 1 hour)
    pub fn validate_timestamp(env: &Env, timestamp: u64) -> bool {
        let current_time = env.ledger().timestamp();
        let one_hour = 3600; // seconds
        
        // Reject future timestamps
        if timestamp > current_time {
            return false;
        }
        
        // Check if timestamp is within the last hour
        current_time - timestamp <= one_hour
    }

    /// Get Chainlink price feed for a crop type
    pub fn get_chainlink_feed(env: &Env, crop_type: &String) -> Result<ChainlinkPriceFeed, StabilizationError> {
        let feed_key = DataKey::ChainlinkFeed(crop_type.clone());
        if !env.storage().persistent().has(&feed_key) {
            return Err(StabilizationError::ChainlinkFeedNotFound);
        }
        
        let feed: ChainlinkPriceFeed = env.storage().persistent().get(&feed_key).unwrap();
        Ok(feed)
    }

    /// Get Chainlink price data for a crop type
    pub fn get_chainlink_price_data(env: &Env, crop_type: &String) -> Result<ChainlinkPriceData, StabilizationError> {
        let price_key = DataKey::ChainlinkPrice(crop_type.clone());
        if !env.storage().persistent().has(&price_key) {
            return Err(StabilizationError::PriceDataNotAvailable);
        }
        
        let price_data: ChainlinkPriceData = env.storage().persistent().get(&price_key).unwrap();
        Ok(price_data)
    }

    /// Validate Chainlink price response
    pub fn validate_chainlink_response(
        env: &Env,
        price: i128,
        timestamp: u64,
        round_id: u64,
        staleness_period: u64,
    ) -> Result<bool, StabilizationError> {
        // Validate price is positive
        if price <= 0 {
            return Err(StabilizationError::InvalidChainlinkResponse);
        }
        
        // Validate timestamp is not in the future
        let current_time = env.ledger().timestamp();
        if timestamp > current_time {
            return Err(StabilizationError::InvalidChainlinkResponse);
        }
        
        // Check if data is stale (older than staleness period)
        if current_time - timestamp > staleness_period {
            return Err(StabilizationError::StalePriceData);
        }
        
        // Validate round_id is positive
        if round_id == 0 {
            return Err(StabilizationError::InvalidChainlinkResponse);
        }
        
        Ok(true)
    }

    /// Convert price from Chainlink decimals to standard format
    pub fn convert_chainlink_price(price: i128, decimals: u32) -> Result<i128, StabilizationError> {
        // Chainlink prices are typically returned with 8 decimals
        // Convert to standard format (2 decimals for USD)
        if decimals > 8 {
            return Err(StabilizationError::InvalidChainlinkResponse);
        }
        
        let conversion_factor = 10_i128.pow(8 - decimals);
        let converted_price = price
            .checked_mul(conversion_factor)
            .ok_or(StabilizationError::InvalidInput)?;
        
        Ok(converted_price)
    }

    /// Check if Chainlink feed is registered and active
    pub fn is_chainlink_feed_active(env: &Env, crop_type: &String) -> Result<bool, StabilizationError> {
        let feed = Self::get_chainlink_feed(env, crop_type)?;
        Ok(feed.active)
    }
}