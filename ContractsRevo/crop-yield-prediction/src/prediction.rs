use soroban_sdk::{Env, String, Vec, BytesN, symbol_short, Map};
use crate::{YieldPrediction, Crop, utils};

const PREDICTIONS: symbol_short!("PRED") = symbol_short!("PRED");
const CROPS: symbol_short!("CROPS") = symbol_short!("CROPS");

/// Generate a yield prediction based on oracle data
pub fn generate_prediction(
    env: &Env,
    crop_id: BytesN<32>,
    region: String,
    weather_data: Vec<i128>,
    soil_data: Vec<i128>,
) -> BytesN<32> {
    // Validate input data
    utils::validate_weather_data(&weather_data);
    utils::validate_soil_data(&soil_data);

    // Get crop information
    let crop = get_crop(env, &crop_id)
        .expect("Crop not registered");

    // Calculate prediction using historical data and current conditions
    let predicted_yield = calculate_yield_prediction(&crop, &weather_data, &soil_data);

    // Generate prediction ID
    let prediction_id = utils::generate_prediction_id(env, &crop_id, &region);

    // Create off-chain data hash
    let off_chain_data = create_off_chain_data(&weather_data, &soil_data);
    let data_hash = utils::hash_data(&off_chain_data);

    // Create prediction
    let prediction = YieldPrediction {
        prediction_id: prediction_id.clone(),
        crop_id,
        region: region.clone(),
        predicted_yield,
        data_hash,
        timestamp: env.ledger().timestamp(),
    };

    // Store prediction
    let mut predictions: Map<BytesN<32>, YieldPrediction> = env
        .storage()
        .persistent()
        .get(&PREDICTIONS)
        .unwrap_or(Map::new(env));
    
    predictions.set(prediction_id.clone(), prediction);
    env.storage().persistent().set(&PREDICTIONS, &predictions);

    // Store off-chain data (IPFS hash would be returned here)
    store_off_chain_data(&off_chain_data);

    // Emit event
    env.events().publish(
        (symbol_short!("predict"), crop_id),
        (prediction_id.clone(), predicted_yield, region)
    );

    prediction_id
}

/// Register a crop with historical yield data
pub fn register_crop(
    env: &Env,
    crop_id: BytesN<32>,
    name: String,
    historical_yields: Vec<i128>,
) {
    utils::validate_historical_yields(&historical_yields);

    let crop = Crop {
        crop_id: crop_id.clone(),
        name: name.clone(),
        historical_yields,
    };

    let mut crops: Map<BytesN<32>, Crop> = env
        .storage()
        .persistent()
        .get(&CROPS)
        .unwrap_or(Map::new(env));
    
    crops.set(crop_id.clone(), crop);
    env.storage().persistent().set(&CROPS, &crops);

    // Emit event
    env.events().publish(
        (symbol_short!("crop_reg"), crop_id),
        name
    );
}

/// Get a specific yield prediction
pub fn get_prediction(env: &Env, prediction_id: BytesN<32>) -> Option<YieldPrediction> {
    let predictions: Map<BytesN<32>, YieldPrediction> = env
        .storage()
        .persistent()
        .get(&PREDICTIONS)
        .unwrap_or(Map::new(env));
    
    predictions.get(prediction_id)
}

/// Get crop information
pub fn get_crop(env: &Env, crop_id: &BytesN<32>) -> Option<Crop> {
    let crops: Map<BytesN<32>, Crop> = env
        .storage()
        .persistent()
        .get(&CROPS)
        .unwrap_or(Map::new(env));
    
    crops.get(crop_id.clone())
}

// Private helper functions

fn calculate_yield_prediction(
    crop: &Crop,
    weather_data: &Vec<i128>,
    soil_data: &Vec<i128>,
) -> i128 {
    // Calculate historical average
    let historical_avg = if crop.historical_yields.len() > 0 {
        crop.historical_yields.iter().sum::<i128>() / crop.historical_yields.len() as i128
    } else {
        0
    };

    // Weather impact factor (simplified model)
    let weather_factor = calculate_weather_impact(weather_data);
    
    // Soil impact factor (simplified model)
    let soil_factor = calculate_soil_impact(soil_data);

    // Combined prediction (baseline + adjustments)
    let predicted_yield = historical_avg + (historical_avg * weather_factor / 100) + (historical_avg * soil_factor / 100);
    
    // Ensure non-negative yield
    if predicted_yield < 0 { 0 } else { predicted_yield }
}

fn calculate_weather_impact(weather_data: &Vec<i128>) -> i128 {
    // Simplified weather impact calculation
    // weather_data: [temperature, humidity, rainfall, sunshine_hours]
    if weather_data.len() < 4 {
        return 0;
    }

    let temperature = weather_data.get(0).unwrap_or(0);
    let humidity = weather_data.get(1).unwrap_or(0);
    let rainfall = weather_data.get(2).unwrap_or(0);
    let sunshine = weather_data.get(3).unwrap_or(0);

    // Optimal ranges (simplified)
    let temp_impact = if temperature >= 18 && temperature <= 25 { 10 } else { -5 };
    let humidity_impact = if humidity >= 60 && humidity <= 80 { 5 } else { -3 };
    let rain_impact = if rainfall >= 500 && rainfall <= 1000 { 15 } else { -10 };
    let sun_impact = if sunshine >= 6 && sunshine <= 8 { 8 } else { -5 };

    temp_impact + humidity_impact + rain_impact + sun_impact
}

fn calculate_soil_impact(soil_data: &Vec<i128>) -> i128 {
    // Simplified soil impact calculation
    // soil_data: [ph_level, nitrogen, phosphorus, potassium, organic_matter]
    if soil_data.len() < 5 {
        return 0;
    }

    let ph = soil_data.get(0).unwrap_or(0);
    let nitrogen = soil_data.get(1).unwrap_or(0);
    let phosphorus = soil_data.get(2).unwrap_or(0);
    let potassium = soil_data.get(3).unwrap_or(0);
    let organic_matter = soil_data.get(4).unwrap_or(0);

    // Optimal ranges (simplified, values scaled by 10)
    let ph_impact = if ph >= 60 && ph <= 75 { 8 } else { -4 };
    let n_impact = if nitrogen >= 20 && nitrogen <= 40 { 12 } else { -6 };
    let p_impact = if phosphorus >= 15 && phosphorus <= 30 { 8 } else { -4 };
    let k_impact = if potassium >= 100 && potassium <= 200 { 10 } else { -5 };
    let om_impact = if organic_matter >= 25 && organic_matter <= 50 { 15 } else { -8 };

    ph_impact + n_impact + p_impact + k_impact + om_impact
}

fn create_off_chain_data(weather_data: &Vec<i128>, soil_data: &Vec<i128>) -> Vec<i128> {
    let mut combined_data = Vec::new(&env);
    
    // Add weather data
    for i in 0..weather_data.len() {
        combined_data.push_back(weather_data.get(i).unwrap_or(0));
    }
    
    // Add soil data
    for i in 0..soil_data.len() {
        combined_data.push_back(soil_data.get(i).unwrap_or(0));
    }
    
    combined_data
}

fn store_off_chain_data(data: &Vec<i128>) {
    // In production, this would integrate with IPFS
    // For now, we'll just log the data size
    // The hash is what gets stored on-chain
}
