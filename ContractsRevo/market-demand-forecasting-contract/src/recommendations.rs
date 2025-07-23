use crate::error::ContractError;
use crate::storage::{self, Product};
use soroban_sdk::{BytesN, Env, Map, String, Vec};

// A constant defining the number of seconds in a day.
const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

/// Generates crop planting recommendations for a given region based on demand forecasts
/// within the specified number of days.
pub fn generate_recommendation(
    env: &Env,
    region: String,
    time_window_days: u64,
) -> Result<Vec<Product>, ContractError> {
    let forecast_ids = storage::get_region_forecast_ids(env, &region);

    if forecast_ids.is_empty() {
        return Err(ContractError::RegionNotFound);
    }

    let time_window_seconds = time_window_days * SECONDS_IN_DAY;
    let current_time = env.ledger().timestamp();
    let cutoff_time = current_time.saturating_sub(time_window_seconds);

    // Map to store a list of recent demand values for each product.
    // Key: product_id, Value: Vec of recent demands.
    let mut recent_product_demands: Map<BytesN<32>, Vec<i128>> = Map::new(env);

    for id in forecast_ids.iter() {
        if let Ok(forecast) = storage::get_forecast(env, &id) {
            // Only consider forecasts within the time window.
            if forecast.timestamp >= cutoff_time {
                let mut demands = recent_product_demands
                    .get(forecast.product_id.clone())
                    .unwrap_or_else(|| Vec::new(env));
                demands.push_back(forecast.predicted_demand);
                recent_product_demands.set(forecast.product_id.clone(), demands);
            }
        }
    }

    if recent_product_demands.is_empty() {
        // No recent forecasts found for this region.
        return Ok(Vec::new(env));
    }

    // Calculate the average demand for each product and store it for sorting.
    let mut averaged_demands = Vec::new(env);
    for (product_id, demands) in recent_product_demands.iter() {
        if !demands.is_empty() {
            let sum: i128 = demands.iter().sum();
            let count = demands.len() as i128;
            let average = sum / count;
            averaged_demands.push_back((product_id, average));
        }
    }

    // Sort the products by their average demand in descending order using Insertion Sort.
    let len = averaged_demands.len();
    if len > 1 {
        for i in 1..len {
            let key = averaged_demands.get(i).unwrap();
            let mut j = i - 1;

            // Move elements of averaged_demands[0..i-1], that are less than key.demand,
            // to one position ahead of their current position.
            while averaged_demands.get(j).unwrap().1 < key.1 {
                let val = averaged_demands.get(j).unwrap();
                averaged_demands.set(j + 1, val);
                if j == 0 {
                    // Break the loop if we've reached the beginning of the vec.
                    // A 'j' of 0 will become max_u32 on the next pass, causing an out-of-bounds error.
                    break;
                }
                j -= 1;
            }

            // Correctly place the key after the last element that was smaller.
            if averaged_demands.get(j).unwrap().1 < key.1 {
                 averaged_demands.set(j, key);
            } else {
                 averaged_demands.set(j + 1, key);
            }
        }
    }


    // Fetch the full product data for the top recommendations.
    let mut recommendations = Vec::new(env);
    let limit = 5_u32.min(averaged_demands.len());

    for i in 0..limit {
        let (product_id, _) = averaged_demands.get(i).unwrap();
        if let Ok(product) = storage::get_product(env, &product_id) {
            recommendations.push_back(product);
        }
    }

    Ok(recommendations)
}
