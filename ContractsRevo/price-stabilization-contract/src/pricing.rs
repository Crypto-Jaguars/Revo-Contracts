use crate::datatype::{DataKey, PriceData, StabilizationError, StabilizationFund, ChainlinkPriceFeed, ChainlinkPriceData};
use crate::PriceStabilizationContractArgs;
use crate::interface::PriceMonitoring;
use crate::{PriceStabilizationContract, PriceStabilizationContractClient};
use soroban_sdk::{contractimpl, Address, BytesN, Env, String};

#[contractimpl]
impl PriceMonitoring for PriceStabilizationContract {
    fn register_price_oracle(
        env: Env,
        admin: Address,
        oracle_address: Address,
        crop_type: String,
    ) -> Result<(), StabilizationError> {
        // Verify admin authorization
        admin.require_auth();
        
        // Verify admin is the contract admin
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(StabilizationError::Unauthorized);
        }
        
        // Validate inputs
        if crop_type.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Register the oracle for the crop type
        let oracle_key = DataKey::PriceOracle(crop_type.clone());
        env.storage().persistent().set(&oracle_key, &oracle_address);
        
        Ok(())
    }

    fn update_market_price(
        env: Env,
        oracle: Address,
        crop_type: String,
        price: i128,
        timestamp: u64,
    ) -> Result<(), StabilizationError> {
        // Verify oracle authorization
        oracle.require_auth();
        
        // Validate inputs
        if price <= 0 || crop_type.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Validate timestamp using existing utility
        if !crate::utils::PriceUtils::validate_timestamp(&env, timestamp) {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Check if oracle is registered for this crop type
        let oracle_key = DataKey::PriceOracle(crop_type.clone());
        if !env.storage().persistent().has(&oracle_key) {
            return Err(StabilizationError::OracleNotRegistered);
        }
        
        // Verify oracle is authorized for this crop type
        let registered_oracle: Address = env.storage().persistent().get(&oracle_key).unwrap();
        if oracle != registered_oracle {
            return Err(StabilizationError::Unauthorized);
        }
        
        // Create price data
        let price_data = PriceData {
            price,
            timestamp,
            oracle: oracle.clone(),
        };
        
        // Store the price data
        let price_key = DataKey::MarketPrice(crop_type.clone());
        env.storage().persistent().set(&price_key, &price_data);
        
        Ok(())
    }

    fn get_market_price(
        env: Env,
        crop_type: String,
    ) -> Result<(i128, u64), StabilizationError> {
        // Validate inputs
        if crop_type.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Check if price data exists for this crop type
        let price_key = DataKey::MarketPrice(crop_type.clone());
        if !env.storage().persistent().has(&price_key) {
            return Err(StabilizationError::PriceDataNotAvailable);
        }
        
        // Get the price data
        let price_data: PriceData = env.storage().persistent().get(&price_key).unwrap();
        
        Ok((price_data.price, price_data.timestamp))
    }

    fn check_price_threshold(
        env: Env,
        fund_id: BytesN<32>,
    ) -> Result<bool, StabilizationError> {
        // Check if fund exists
        let fund_key = DataKey::Fund(fund_id.clone());
        if !env.storage().persistent().has(&fund_key) {
            return Err(StabilizationError::FundNotFound);
        }
        
        // Get the fund
        let fund: StabilizationFund = env.storage().persistent().get(&fund_key).unwrap();
        
        // Check if price data exists for this crop type
        let price_key = DataKey::MarketPrice(fund.crop_type.clone());
        if !env.storage().persistent().has(&price_key) {
            return Err(StabilizationError::PriceDataNotAvailable);
        }
        
        // Get the price data
        let price_data: PriceData = env.storage().persistent().get(&price_key).unwrap();
        
        // Check if current price is below threshold
        let below_threshold = price_data.price < fund.price_threshold;
        
        Ok(below_threshold)
    }

    fn register_chainlink_feed(
        env: Env,
        admin: Address,
        crop_type: String,
        feed_address: Address,
        decimals: u32,
        description: String,
    ) -> Result<(), StabilizationError> {
        // Verify admin authorization
        admin.require_auth();
        
        // Verify admin is the contract admin
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(StabilizationError::Unauthorized);
        }
        
        // Validate inputs
        if crop_type.is_empty() || description.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Check if feed is already registered
        let feed_key = DataKey::ChainlinkFeed(crop_type.clone());
        if env.storage().persistent().has(&feed_key) {
            return Err(StabilizationError::FundAlreadyExists);
        }
        
        // Create Chainlink price feed
        let chainlink_feed = ChainlinkPriceFeed {
            feed_address: feed_address.clone(),
            decimals,
            description,
            crop_type: crop_type.clone(),
            registered_time: env.ledger().timestamp(),
            active: true,
        };
        
        // Store the feed
        env.storage().persistent().set(&feed_key, &chainlink_feed);
        
        Ok(())
    }

    fn get_chainlink_price(
        env: Env,
        crop_type: String,
    ) -> Result<(i128, u64), StabilizationError> {
        // Validate inputs
        if crop_type.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Check if Chainlink feed is registered and active
        if !crate::utils::PriceUtils::is_chainlink_feed_active(&env, &crop_type)? {
            return Err(StabilizationError::ChainlinkFeedNotRegistered);
        }
        
        // Get Chainlink price data
        let price_data = crate::utils::PriceUtils::get_chainlink_price_data(&env, &crop_type)?;
        
        // Convert price to standard format
        let converted_price = crate::utils::PriceUtils::convert_chainlink_price(
            price_data.price,
            price_data.decimals,
        )?;
        
        Ok((converted_price, price_data.timestamp))
    }

    fn update_chainlink_price(
        env: Env,
        oracle: Address,
        crop_type: String,
        price: i128,
        timestamp: u64,
        round_id: u64,
        decimals: u32,
    ) -> Result<(), StabilizationError> {
        // Verify oracle authorization
        oracle.require_auth();
        
        // Validate inputs
        if price <= 0 || crop_type.is_empty() {
            return Err(StabilizationError::InvalidInput);
        }
        
        // Check if Chainlink feed is registered
        let feed = crate::utils::PriceUtils::get_chainlink_feed(&env, &crop_type)?;
        
        // Verify oracle is authorized for this feed
        if oracle != feed.feed_address {
            return Err(StabilizationError::Unauthorized);
        }
        
        // Validate Chainlink response (1 hour staleness period)
        let staleness_period = 3600; // 1 hour
        crate::utils::PriceUtils::validate_chainlink_response(
            &env,
            price,
            timestamp,
            round_id,
            staleness_period,
        )?;
        
        // Create Chainlink price data
        let chainlink_price_data = ChainlinkPriceData {
            price,
            timestamp,
            feed_address: oracle.clone(),
            round_id,
            decimals,
        };
        
        // Store the price data
        let price_key = DataKey::ChainlinkPrice(crop_type.clone());
        env.storage().persistent().set(&price_key, &chainlink_price_data);
        
        Ok(())
    }
}