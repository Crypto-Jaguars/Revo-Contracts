use soroban_sdk::{Env, Address, String, Map, symbol_short};

const ORACLES: symbol_short!("ORACLES") = symbol_short!("ORACLES");

#[derive(Clone)]
pub struct OracleConfig {
    pub address: Address,
    pub data_type: String,
    pub is_active: bool,
    pub last_update: u64,
}

/// Update oracle data source
pub fn update_data_source(env: &Env, oracle_address: Address, data_type: String) {
    let mut oracles: Map<String, OracleConfig> = env
        .storage()
        .persistent()
        .get(&ORACLES)
        .unwrap_or(Map::new(env));

    let config = OracleConfig {
        address: oracle_address.clone(),
        data_type: data_type.clone(),
        is_active: true,
        last_update: env.ledger().timestamp(),
    };

    oracles.set(data_type.clone(), config);
    env.storage().persistent().set(&ORACLES, &oracles);

    // Emit event
    env.events().publish(
        (symbol_short!("oracle"), data_type),
        oracle_address
    );
}

/// Get oracle configuration
pub fn get_oracle_config(env: &Env, data_type: String) -> Option<OracleConfig> {
    let oracles: Map<String, OracleConfig> = env
        .storage()
        .persistent()
        .get(&ORACLES)
        .unwrap_or(Map::new(env));
    
    oracles.get(data_type)
}

/// Validate oracle data integrity
pub fn validate_oracle_data(env: &Env, data_type: String, data: &Vec<i128>) -> bool {
    let config = get_oracle_config(env, data_type);
    
    match config {
        Some(oracle_config) => {
            // Check if oracle is active
            if !oracle_config.is_active {
                return false;
            }
            
            // Check data freshness (within 24 hours)
            let current_time = env.ledger().timestamp();
            let data_age = current_time - oracle_config.last_update;
            if data_age > 86400 { // 24 hours in seconds
                return false;
            }
            
            // Validate data format based on type
            match oracle_config.data_type.as_str() {
                "weather" => validate_weather_format(data),
                "soil" => validate_soil_format(data),
                _ => true, // Unknown type, accept for now
            }
        }
        None => false, // No oracle configured
    }
}

fn validate_weather_format(data: &Vec<i128>) -> bool {
    // Weather data should have: [temperature, humidity, rainfall, sunshine_hours]
    if data.len() != 4 {
        return false;
    }
    
    let temp = data.get(0).unwrap_or(0);
    let humidity = data.get(1).unwrap_or(0);
    let rainfall = data.get(2).unwrap_or(0);
    let sunshine = data.get(3).unwrap_or(0);
    
    // Basic range validation
    temp >= -50 && temp <= 50 &&      // Temperature in Celsius
    humidity >= 0 && humidity <= 100 &&  // Humidity percentage
    rainfall >= 0 && rainfall <= 5000 && // Annual rainfall in mm
    sunshine >= 0 && sunshine <= 24      // Daily sunshine hours
}

fn validate_soil_format(data: &Vec<i128>) -> bool {
    // Soil data should have: [ph_level, nitrogen, phosphorus, potassium, organic_matter]
    if data.len() != 5 {
        return false;
    }
    
    let ph = data.get(0).unwrap_or(0);
    let nitrogen = data.get(1).unwrap_or(0);
    let phosphorus = data.get(2).unwrap_or(0);
    let potassium = data.get(3).unwrap_or(0);
    let organic_matter = data.get(4).unwrap_or(0);
    
    // Basic range validation (scaled by 10)
    ph >= 0 && ph <= 140 &&           // pH 0-14
    nitrogen >= 0 && nitrogen <= 1000 &&  // Nitrogen content
    phosphorus >= 0 && phosphorus <= 500 && // Phosphorus content
    potassium >= 0 && potassium <= 2000 &&  // Potassium content
    organic_matter >= 0 && organic_matter <= 1000 // Organic matter percentage
}
