use soroban_sdk::{contracterror, contracttype, BytesN, String, Vec};

#[derive(Clone, Debug)]
#[contracttype]
pub struct YieldPrediction {
    pub prediction_id: BytesN<32>,
    pub crop_id: BytesN<32>,
    pub region: String,
    pub predicted_yield: i128,
    pub data_hash: BytesN<32>, // Hash of off-chain data
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Crop {
    pub crop_id: BytesN<32>,
    pub name: String,
    pub historical_yields: Vec<i128>,
}

#[derive(Clone)]
#[contracttype]
pub struct DataSource {
    pub weather_data: String,
    pub soil_data: String,
    pub temperature: i32,
    pub humidity: i32,
    pub rainfall: i32,
}

#[derive(Clone)]
#[contracttype]
pub struct YieldReport {
    pub crop_name: String,
    pub region: String,
    pub predicted_yield: i128,
    pub recommendations: String,
    pub report_date: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct MarketInsight {
    pub crop_id: String,
    pub region: String,
    pub expected_supply: i128,
    pub price_trend: String,
    pub buying_recommendation: String,
}

///////////////////////////////////////////////////////////
//////            DataKeys                         ///////
/////////////////////////////////////////////////////////

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CROPS,
    ADMIN,
    PREDICTIONS,
}

/////////////////////////////////////////////////////
///////////    ERROR                            ////
///////////////////////////////////////////////////

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[repr(u32)]
pub enum CropYieldError {
    Unauthorized = 1,
    InvalidInput = 2,
    CropNotFound = 3,
    PredictionNotFound = 4,
    ContractNotInitialized = 5,
    InvalidYieldData = 6,
    DataProcessingError = 7,
}
