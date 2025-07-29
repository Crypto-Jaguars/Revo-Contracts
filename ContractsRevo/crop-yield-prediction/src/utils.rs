use crate::types::{Crop, DataSource};
use soroban_sdk::{Bytes, BytesN, Env};

pub fn generate_prediction_id(env: &Env, crop_id: &BytesN<32>) -> BytesN<32> {
    let mut combined = Bytes::new(env);

    combined.append(&Bytes::from_array(env, &crop_id.to_array()));
    combined.append(&Bytes::from_array(
        env,
        &env.ledger().timestamp().to_be_bytes(),
    ));

    env.crypto().sha256(&combined).into()
}

pub fn hash_data_source(env: &Env, data_source: &DataSource) -> BytesN<32> {
    let mut combined = Bytes::new(env);

    combined.append(&Bytes::from_array(
        env,
        &data_source.temperature.to_be_bytes(),
    ));
    combined.append(&Bytes::from_array(env, &data_source.humidity.to_be_bytes()));
    combined.append(&Bytes::from_array(env, &data_source.rainfall.to_be_bytes()));

    // Hash the combined bytes
    env.crypto().sha256(&combined).into()
}

pub fn calculate_yield_prediction(crop: &Crop, data_source: &DataSource) -> i128 {
    if crop.historical_yields.is_empty() {
        return 0;
    }

    // Calculate average historical yield
    let mut total: i128 = 0;
    for yield_val in crop.historical_yields.iter() {
        total += yield_val;
    }
    let avg_yield = total / crop.historical_yields.len() as i128;

    // Simple weather adjustment factors
    let temperature_factor = if data_source.temperature > 25 && data_source.temperature < 35 {
        110 // Good temperature range
    } else {
        90 // Suboptimal temperature
    };

    let humidity_factor = if data_source.humidity > 40 && data_source.humidity < 70 {
        110 // Good humidity range
    } else {
        85 // Suboptimal humidity
    };

    let rainfall_factor = if data_source.rainfall > 50 && data_source.rainfall < 200 {
        115 // Good rainfall
    } else if data_source.rainfall < 30 {
        70 // Too little rain
    } else {
        80 // Too much rain
    };

    // Calculate adjusted yield
    let adjustment = (temperature_factor + humidity_factor + rainfall_factor) / 3;
    (avg_yield * adjustment) / 100
}
