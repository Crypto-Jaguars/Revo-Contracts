use soroban_sdk::{Env, String, Vec, BytesN, Map, symbol_short};
use crate::{YieldPrediction, prediction, utils};

#[derive(Clone)]
pub struct FarmerRecommendation {
    pub crop_id: BytesN<32>,
    pub region: String,
    pub recommended_planting_date: u64,
    pub expected_yield: i128,
    pub risk_level: String,
    pub recommendations: Vec<String>,
}

#[derive(Clone)]
pub struct BuyerInsight {
    pub crop_id: BytesN<32>,
    pub region: String,
    pub predicted_supply: i128,
    pub price_trend: String,
    pub availability_window: u64,
    pub quality_score: i128,
}

const PREDICTIONS: symbol_short!("PRED") = symbol_short!("PRED");

/// List predictions filtered by region or crop
pub fn list_predictions(
    env: &Env,
    filter_type: String,
    filter_value: String,
) -> Vec<YieldPrediction> {
    let predictions: Map<BytesN<32>, YieldPrediction> = env
        .storage()
        .persistent()
        .get(&PREDICTIONS)
        .unwrap_or(Map::new(env));

    let mut filtered_predictions = Vec::new(env);

    // Iterate through all predictions and filter
    let keys = predictions.keys();
    for i in 0..keys.len() {
        if let Some(key) = keys.get(i) {
            if let Some(prediction) = predictions.get(key) {
                let matches = match filter_type.as_str() {
                    "region" => prediction.region == filter_value,
                    "crop" => {
                        // Convert crop_id to string for comparison (simplified)
                        true // In production, would properly convert and compare
                    }
                    _ => true, // No filter, include all
                };

                if matches {
                    filtered_predictions.push_back(prediction);
                }
            }
        }
    }

    filtered_predictions
}

/// Generate farmer recommendations based on predictions
pub fn get_farmer_recommendations(
    env: &Env,
    crop_id: BytesN<32>,
    region: String,
) -> FarmerRecommendation {
    // Get recent predictions for this crop and region
    let predictions = list_predictions(env, String::from_str(env, "region"), region.clone());
    
    let mut total_yield = 0i128;
    let mut prediction_count = 0u32;
    let current_time = env.ledger().timestamp();
    
    // Analyze recent predictions (last 30 days)
    for i in 0..predictions.len() {
        if let Some(prediction) = predictions.get(i) {
            if prediction.crop_id == crop_id && 
               current_time - prediction.timestamp <= 2592000 { // 30 days
                total_yield += prediction.predicted_yield;
                prediction_count += 1;
            }
        }
    }

    let expected_yield = if prediction_count > 0 {
        total_yield / prediction_count as i128
    } else {
        0
    };

    // Calculate risk level
    let risk_level = assess_risk_level(expected_yield, &predictions);

    // Generate recommendations
    let recommendations = generate_farming_recommendations(expected_yield, &risk_level);

    // Calculate optimal planting date (simplified)
    let recommended_planting_date = current_time + 2592000; // 30 days from now

    FarmerRecommendation {
        crop_id,
        region,
        recommended_planting_date,
        expected_yield,
        risk_level,
        recommendations,
    }
}

/// Generate buyer insights for procurement decisions
pub fn get_buyer_insights(
    env: &Env,
    crop_id: BytesN<32>,
    region: String,
    timeframe: u64,
) -> BuyerInsight {
    let predictions = list_predictions(env, String::from_str(env, "region"), region.clone());
    let current_time = env.ledger().timestamp();
    
    let mut total_supply = 0i128;
    let mut prediction_count = 0u32;
    
    // Analyze predictions within timeframe
    for i in 0..predictions.len() {
        if let Some(prediction) = predictions.get(i) {
            if prediction.crop_id == crop_id &&
               prediction.timestamp >= current_time &&
               prediction.timestamp <= current_time + timeframe {
                total_supply += prediction.predicted_yield;
                prediction_count += 1;
            }
        }
    }

    let predicted_supply = if prediction_count > 0 {
        total_supply
    } else {
        0
    };

    // Analyze price trend (simplified)
    let price_trend = analyze_price_trend(predicted_supply, prediction_count);

    // Calculate availability window
    let availability_window = calculate_availability_window(&predictions, current_time);

    // Quality score based on prediction confidence
    let quality_score = calculate_quality_score(prediction_count, predicted_supply);

    BuyerInsight {
        crop_id,
        region,
        predicted_supply,
        price_trend,
        availability_window,
        quality_score,
    }
}

// Private helper functions

fn assess_risk_level(expected_yield: i128, predictions: &Vec<YieldPrediction>) -> String {
    if expected_yield == 0 {
        return String::from_str(&Env::default(), "HIGH");
    }

    // Calculate yield variance
    let mut variance = 0i128;
    let mut count = 0u32;
    
    for i in 0..predictions.len() {
        if let Some(prediction) = predictions.get(i) {
            let diff = prediction.predicted_yield - expected_yield;
            variance += diff * diff;
            count += 1;
        }
    }

    if count == 0 {
        return String::from_str(&Env::default(), "MEDIUM");
    }

    let avg_variance = variance / count as i128;
    
    if avg_variance < 1000000 { // Low variance
        String::from_str(&Env::default(), "LOW")
    } else if avg_variance < 5000000 { // Medium variance
        String::from_str(&Env::default(), "MEDIUM")
    } else { // High variance
        String::from_str(&Env::default(), "HIGH")
    }
}

fn generate_farming_recommendations(expected_yield: i128, risk_level: &String) -> Vec<String> {
    let mut recommendations = Vec::new(&Env::default());
    
    if expected_yield < 1000 {
        recommendations.push_back(String::from_str(&Env::default(), "Consider soil improvement"));
        recommendations.push_back(String::from_str(&Env::default(), "Evaluate irrigation systems"));
    }
    
    match risk_level.as_str() {
        "HIGH" => {
            recommendations.push_back(String::from_str(&Env::default(), "Consider crop insurance"));
            recommendations.push_back(String::from_str(&Env::default(), "Diversify crop selection"));
        }
        "MEDIUM" => {
            recommendations.push_back(String::from_str(&Env::default(), "Monitor weather patterns"));
            recommendations.push_back(String::from_str(&Env::default(), "Prepare contingency plans"));
        }
        "LOW" => {
            recommendations.push_back(String::from_str(&Env::default(), "Optimal conditions expected"));
            recommendations.push_back(String::from_str(&Env::default(), "Consider expanding acreage"));
        }
        _ => {}
    }
    
    recommendations
}

fn analyze_price_trend(predicted_supply: i128, prediction_count: u32) -> String {
    if prediction_count == 0 {
        return String::from_str(&Env::default(), "UNKNOWN");
    }

    // Simplified price trend analysis
    if predicted_supply > 10000 {
        String::from_str(&Env::default(), "DECREASING") // High supply, lower prices
    } else if predicted_supply > 5000 {
        String::from_str(&Env::default(), "STABLE")
    } else {
        String::from_str(&Env::default(), "INCREASING") // Low supply, higher prices
    }
}

fn calculate_availability_window(predictions: &Vec<YieldPrediction>, current_time: u64) -> u64 {
    // Find the earliest harvest time (simplified)
    current_time + 15552000 // 6 months from now (typical growing season)
}

fn calculate_quality_score(prediction_count: u32, predicted_supply: i128) -> i128 {
    // Quality score based on data availability and supply predictions
    let data_score = if prediction_count > 10 { 50 } else { prediction_count as i128 * 5 };
    let supply_score = if predicted_supply > 0 { 50 } else { 0 };
    
    data_score + supply_score
}
