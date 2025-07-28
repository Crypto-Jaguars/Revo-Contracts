use soroban_sdk::{Env, Address, BytesN, Vec, String, symbol_short, crypto::Hash};

const ADMIN: symbol_short!("ADMIN") = symbol_short!("ADMIN");

/// Set contract administrator
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

/// Get contract administrator
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&ADMIN)
}

/// Require caller to be admin
pub fn require_admin(env: &Env, caller: &Address) {
    let admin = get_admin(env).expect("Admin not set");
    if admin != *caller {
        panic!("Unauthorized: Admin required");
    }
}

/// Generate unique prediction ID
pub fn generate_prediction_id(env: &Env, crop_id: &BytesN<32>, region: &String) -> BytesN<32> {
    let timestamp = env.ledger().timestamp();
    let sequence = env.ledger().sequence();
    
    // Create unique identifier by hashing crop_id + region + timestamp + sequence
    let mut data = Vec::new(env);
    
    // Add crop_id bytes
    for i in 0..32 {
        data.push_back(crop_id.get_byte(i).unwrap_or(0) as i128);
    }
    
    // Add region bytes (simplified)
    let region_bytes = region.as_bytes();
    for i in 0..region_bytes.len().min(32) {
        data.push_back(region_bytes[i] as i128);
    }
    
    // Add timestamp and sequence
    data.push_back(timestamp as i128);
    data.push_back(sequence as i128);
    
    hash_data(&data)
}

/// Hash data for integrity verification
pub fn hash_data(data: &Vec<i128>) -> BytesN<32> {
    // Convert data to bytes for hashing
    let mut bytes = Vec::new(&Env::default());
    for i in 0..data.len() {
        let value = data.get(i).unwrap_or(0);
        // Convert i128 to bytes (simplified)
        for j in 0..16 {
            bytes.push_back(((value >> (j * 8)) & 0xFF) as u8);
        }
    }
    
    // Use Stellar's crypto hash
    let env = Env::default();
    env.crypto().sha256(&bytes)
}

/// Validate weather data format and ranges
pub fn validate_weather_data(weather_data: &Vec<i128>) {
    if weather_data.len() != 4 {
        panic!("Weather data must contain exactly 4 elements");
    }
    
    let temperature = weather_data.get(0).unwrap_or(0);
    let humidity = weather_data.get(1).unwrap_or(0);
    let rainfall = weather_data.get(2).unwrap_or(0);
    let sunshine = weather_data.get(3).unwrap_or(0);
    
    if temperature < -50 || temperature > 50 {
        panic!("Invalid temperature range");
    }
    
    if humidity < 0 || humidity > 100 {
        panic!("Invalid humidity range");
    }
    
    if rainfall < 0 || rainfall > 5000 {
        panic!("Invalid rainfall range");
    }
    
    if sunshine < 0 || sunshine > 24 {
        panic!("Invalid sunshine hours range");
    }
}

/// Validate soil data format and ranges
pub fn validate_soil_data(soil_data: &Vec<i128>) {
    if soil_data.len() != 5 {
        panic!("Soil data must contain exactly 5 elements");
    }
    
    let ph = soil_data.get(0).unwrap_or(0);
    let nitrogen = soil_data.get(1).unwrap_or(0);
    let phosphorus = soil_data.get(2).unwrap_or(0);
    let potassium = soil_data.get(3).unwrap_or(0);
    let organic_matter = soil_data.get(4).unwrap_or(0);
    
    if ph < 0 || ph > 140 {
        panic!("Invalid pH range");
    }
    
    if nitrogen < 0 || nitrogen > 1000 {
        panic!("Invalid nitrogen range");
    }
    
    if phosphorus < 0 || phosphorus > 500 {
        panic!("Invalid phosphorus range");
    }
    
    if potassium < 0 || potassium > 2000 {
        panic!("Invalid potassium range");
    }
    
    if organic_matter < 0 || organic_matter > 1000 {
        panic!("Invalid organic matter range");
    }
}

/// Validate historical yields data
pub fn validate_historical_yields(yields: &Vec<i128>) {
    if yields.len() == 0 {
        panic!("Historical yields cannot be empty");
    }
    
    if yields.len() > 50 {
        panic!("Too many historical yield entries");
    }
    
    for i in 0..yields.len() {
        let yield_value = yields.get(i).unwrap_or(0);
        if yield_value < 0 {
            panic!("Yield values must be non-negative");
        }
    }
}

/// Data structures for the contract
#[derive(Clone)]
pub struct YieldPrediction {
    pub prediction_id: BytesN<32>,
    pub crop_id: BytesN<32>,
    pub region: String,
    pub predicted_yield: i128,
    pub data_hash: BytesN<32>,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct Crop {
    pub crop_id: BytesN<32>,
    pub name: String,
    pub historical_yields: Vec<i128>,
}
