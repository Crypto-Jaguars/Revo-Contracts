use crate::datatype::StabilizationError;
use soroban_sdk::{Address, Env, Map, String, Vec, BytesN};

#[allow(dead_code)]
pub trait FundManagement {
    /// Initialize a new stabilization fund with price thresholds
    fn create_fund(
        env: Env,
        admin: Address,
        fund_name: String,
        price_threshold: i128,
        crop_type: String,
    ) -> Result<BytesN<32>, StabilizationError>;

    /// Allow contributions to the fund from farmers or buyers
    fn contribute_fund(
        env: Env,
        contributor: Address,
        fund_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), StabilizationError>;

    /// Retrieve fund balance and contributor details
    fn get_fund_status(
        env: Env,
        fund_id: BytesN<32>,
    ) -> Result<Map<String, i128>, StabilizationError>;

    /// Adjust price thresholds based on market conditions
    fn update_price_threshold(
        env: Env,
        admin: Address,
        fund_id: BytesN<32>,
        new_threshold: i128,
    ) -> Result<(), StabilizationError>;
}

#[allow(dead_code)]
pub trait PriceMonitoring {
    /// Register a price oracle for a specific crop type
    fn register_price_oracle(
        env: Env,
        admin: Address,
        oracle_address: Address,
        crop_type: String,
    ) -> Result<(), StabilizationError>;

    /// Update the current market price from an oracle
    fn update_market_price(
        env: Env,
        oracle: Address,
        crop_type: String,
        price: i128,
        timestamp: u64,
    ) -> Result<(), StabilizationError>;

    /// Get the current market price for a crop type
    fn get_market_price(
        env: Env,
        crop_type: String,
    ) -> Result<(i128, u64), StabilizationError>;

    /// Check if the current price is below the threshold
    fn check_price_threshold(
        env: Env,
        fund_id: BytesN<32>,
    ) -> Result<bool, StabilizationError>;
}

#[allow(dead_code)]
pub trait PayoutDistribution {
    /// Distribute funds to farmers when prices fall below thresholds
    fn trigger_payout(
        env: Env,
        admin: Address,
        fund_id: BytesN<32>,
        farmers: Vec<Address>,
    ) -> Result<Map<Address, i128>, StabilizationError>;

    /// Register a farmer to be eligible for payouts
    fn register_farmer(
        env: Env,
        farmer: Address,
        crop_type: String,
        production_capacity: i128,
    ) -> Result<(), StabilizationError>;

    /// Get the history of payouts for a specific farmer
    fn get_payout_history(
        env: Env,
        farmer: Address,
        fund_id: BytesN<32>,
    ) -> Result<Vec<Map<String, i128>>, StabilizationError>;

    /// Calculate the payout amount for a specific farmer
    fn calculate_payout_amount(
        env: Env,
        farmer: Address,
        fund_id: BytesN<32>,
    ) -> Result<i128, StabilizationError>;
}