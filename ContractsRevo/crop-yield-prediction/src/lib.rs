#![no_std]
use soroban_sdk::{contract, contractimpl, Env, String, Map, Vec, BytesN, Address};

mod prediction;
mod data;
mod reporting;
mod utils;

pub use prediction::*;
pub use data::*;
pub use reporting::*;
pub use utils::*;

#[contract]
pub struct CropYieldPredictionContract;

#[contractimpl]
impl CropYieldPredictionContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address) {
        utils::set_admin(&env, &admin);
    }

    /// Generate a new yield prediction
    pub fn generate_prediction(
        env: Env,
        caller: Address,
        crop_id: BytesN<32>,
        region: String,
        weather_data: Vec<i128>,
        soil_data: Vec<i128>,
    ) -> BytesN<32> {
        caller.require_auth();
        prediction::generate_prediction(&env, crop_id, region, weather_data, soil_data)
    }

    /// Register a new crop with historical data
    pub fn register_crop(
        env: Env,
        caller: Address,
        crop_id: BytesN<32>,
        name: String,
        historical_yields: Vec<i128>,
    ) {
        caller.require_auth();
        prediction::register_crop(&env, crop_id, name, historical_yields);
    }

    /// Get a specific yield prediction
    pub fn get_prediction(env: Env, prediction_id: BytesN<32>) -> Option<YieldPrediction> {
        prediction::get_prediction(&env, prediction_id)
    }

    /// List predictions for a region or crop
    pub fn list_predictions(
        env: Env,
        filter_type: String,
        filter_value: String,
    ) -> Vec<YieldPrediction> {
        reporting::list_predictions(&env, filter_type, filter_value)
    }

    /// Update oracle data source
    pub fn update_data_source(
        env: Env,
        caller: Address,
        oracle_address: Address,
        data_type: String,
    ) {
        caller.require_auth();
        utils::require_admin(&env, &caller);
        data::update_data_source(&env, oracle_address, data_type);
    }

    /// Get farmer recommendations
    pub fn get_farmer_recommendations(
        env: Env,
        crop_id: BytesN<32>,
        region: String,
    ) -> FarmerRecommendation {
        reporting::get_farmer_recommendations(&env, crop_id, region)
    }

    /// Get buyer insights
    pub fn get_buyer_insights(
        env: Env,
        crop_id: BytesN<32>,
        region: String,
        timeframe: u64,
    ) -> BuyerInsight {
        reporting::get_buyer_insights(&env, crop_id, region, timeframe)
    }
}
