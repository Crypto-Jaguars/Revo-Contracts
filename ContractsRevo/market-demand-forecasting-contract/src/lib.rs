#![no_std]

mod data;
mod error;
mod forecasting;
mod recommendations;
mod storage;
mod utils;
mod test;

pub use error::ContractError;
pub use storage::{DemandForecast, Product};

use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, String, Vec,
};

#[contract]
pub struct MarketDemandForecastingContract;

#[contractimpl]
impl MarketDemandForecastingContract {
    // Initializes the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Sets the address of the oracle service responsible for providing market data.
    /// Can only be called by the contract admin.
    pub fn set_oracle(env: Env, admin: Address, oracle: Address) -> Result<(), ContractError> {
        admin.require_auth();
        if !storage::is_admin(&env, &admin) {
            return Err(ContractError::Unauthorized);
        }
        storage::set_oracle(&env, &oracle);
        Ok(())
    }

    // Registers a new agricultural product that can be forecasted.
    pub fn register_product(
        env: Env,
        name: String,
        historical_demand: Vec<i128>,
    ) -> Result<BytesN<32>, ContractError> {
        data::register_product(&env, name, historical_demand)
    }

    /// Generates a new demand forecast for a given product.
    pub fn generate_forecast(
        env: Env,
        oracle: Address,
        product_id: BytesN<32>,
        region: String,
        predicted_demand: i128, // The forecast is provided by the oracle.
        data_hash: BytesN<32>, 
    ) -> Result<BytesN<32>, ContractError> {
        
        oracle.require_auth();
        if storage::get_oracle(&env)? != oracle {
            return Err(ContractError::Unauthorized);
        }
        forecasting::generate_forecast(&env, product_id, region, predicted_demand, data_hash)
    }

    /// Retrieves a specific demand forecast by its ID.
    pub fn get_forecast(env: Env, forecast_id: BytesN<32>) -> Result<DemandForecast, ContractError> {
        storage::get_forecast(&env, &forecast_id)
    }

    /// Retrieves a specific product by its ID.
    pub fn get_product(env: Env, product_id: BytesN<32>) -> Result<Product, ContractError> {
        storage::get_product(&env, &product_id)
    }

    /// Lists all available forecasts, with optional filtering by product or region.
    pub fn list_forecasts(
        env: Env,
        product_id: Option<BytesN<32>>,
        region: Option<String>,
    ) -> Vec<DemandForecast> {
        forecasting::list_forecasts(&env, product_id, region)
    }

    /// Generates a crop planting recommendation for a specific region based on recent demand.
    pub fn generate_recommendation(
        env: Env,
        region: String,
        time_window_days: u64,
    ) -> Result<Vec<Product>, ContractError> {
        recommendations::generate_recommendation(&env, region, time_window_days)
    }
}
