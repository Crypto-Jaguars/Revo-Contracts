use crate::error::ContractError;
use crate::storage::{self, DemandForecast};
use crate::utils::utils;
use soroban_sdk::{BytesN, Env, String, Vec};

/// Generates a new demand forecast for a specific product and region.
/// This function is called by the trusted oracle.
pub fn generate_forecast(
    env: &Env,
    product_id: BytesN<32>,
    region: String,
    predicted_demand: i128, // The forecast provided by the oracle.
    data_hash: BytesN<32>,
) -> Result<BytesN<32>, ContractError> {
    storage::get_product(env, &product_id)?;

    if predicted_demand <= 0 {
        return Err(ContractError::InvalidData);
    }

    // Generate a unique ID for the forecast.
    let forecast_id = utils::generate_id(
        env,
        (
            product_id.clone(),
            region.clone(),
            env.ledger().timestamp(),
        ),
    );

    let forecast = DemandForecast {
        forecast_id: forecast_id.clone(),
        product_id,
        region: region.clone(),
        predicted_demand,
        data_hash,
        timestamp: env.ledger().timestamp(),
    };

    // Store the forecast and index it globally and by region.
    storage::set_forecast(env, &forecast);
    storage::add_forecast_id(env, &forecast_id);
    storage::add_forecast_to_region(env, &region, &forecast_id);

    Ok(forecast_id)
}

/// Lists all forecasts, with optional filtering by product and/or region.
pub fn list_forecasts(
    env: &Env,
    product_id_filter: Option<BytesN<32>>,
    region_filter: Option<String>,
) -> Vec<DemandForecast> {
    let mut forecasts = Vec::new(env);

    let forecast_ids = match region_filter {
        // If a region is specified, fetch only those forecast IDs for efficiency.
        Some(region) => storage::get_region_forecast_ids(env, &region),
        // Otherwise, fetch all forecast IDs.
        None => storage::get_all_forecast_ids(env),
    };

    for id in forecast_ids.iter() {
        if let Ok(forecast) = storage::get_forecast(env, &id) {
            // Apply the product filter if it exists.
            let product_match = match &product_id_filter {
                Some(p_id) => forecast.product_id == *p_id,
                None => true,
            };

            if product_match {
                forecasts.push_back(forecast);
            }
        }
    }

    forecasts
}
