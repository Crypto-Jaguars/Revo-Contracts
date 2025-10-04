use crate::error::ContractError;
use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec};

// --- Data Structures ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Product {
    pub product_id: BytesN<32>,
    pub name: String,
    pub historical_demand: Vec<i128>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemandForecast {
    pub forecast_id: BytesN<32>,
    pub product_id: BytesN<32>,
    pub region: String,
    /// Predicted demand, can be units, kg, etc.
    pub predicted_demand: i128,
    /// Hash of off-chain market data (e.g., from IPFS)
    pub data_hash: BytesN<32>,
    pub timestamp: u64,
}

// --- Storage Keys ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Admin,
    Oracle,
    Product(BytesN<32>),
    Forecast(BytesN<32>),
    AllProducts,
    AllForecasts,
    RegionForecasts(String),
}

// --- Admin and Oracle Management ---

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&StorageKey::Admin, admin);
}

pub fn is_admin(env: &Env, address: &Address) -> bool {
    if !has_admin(env) {
        return false;
    }
    env.storage()
        .instance()
        .get::<_, Address>(&StorageKey::Admin)
        .unwrap()
        == *address
}

pub fn set_oracle(env: &Env, oracle: &Address) {
    env.storage().instance().set(&StorageKey::Oracle, oracle);
}

pub fn get_oracle(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&StorageKey::Oracle)
        .ok_or(ContractError::OracleNotSet)
}

// --- Product Management ---

pub fn get_all_product_ids(env: &Env) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&StorageKey::AllProducts)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_product_id(env: &Env, product_id: &BytesN<32>) {
    let mut all_ids = get_all_product_ids(env);
    all_ids.push_back(product_id.clone());
    env.storage()
        .persistent()
        .set(&StorageKey::AllProducts, &all_ids);
}

pub fn get_product(env: &Env, product_id: &BytesN<32>) -> Result<Product, ContractError> {
    env.storage()
        .persistent()
        .get(&StorageKey::Product(product_id.clone()))
        .ok_or(ContractError::ProductNotFound)
}

pub fn set_product(env: &Env, product: &Product) {
    env.storage()
        .persistent()
        .set(&StorageKey::Product(product.product_id.clone()), product);
}

// --- Forecast Management ---

pub fn get_all_forecast_ids(env: &Env) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&StorageKey::AllForecasts)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_forecast_id(env: &Env, forecast_id: &BytesN<32>) {
    let mut all_ids = get_all_forecast_ids(env);
    all_ids.push_back(forecast_id.clone());
    env.storage()
        .persistent()
        .set(&StorageKey::AllForecasts, &all_ids);
}

pub fn get_forecast(env: &Env, forecast_id: &BytesN<32>) -> Result<DemandForecast, ContractError> {
    env.storage()
        .persistent()
        .get(&StorageKey::Forecast(forecast_id.clone()))
        .ok_or(ContractError::ForecastNotFound)
}

pub fn set_forecast(env: &Env, forecast: &DemandForecast) {
    env.storage().persistent().set(
        &StorageKey::Forecast(forecast.forecast_id.clone()),
        forecast,
    );
}

// --- Region-based Forecast Indexing ---

pub fn get_region_forecast_ids(env: &Env, region: &String) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&StorageKey::RegionForecasts(region.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_forecast_to_region(env: &Env, region: &String, forecast_id: &BytesN<32>) {
    let mut region_ids = get_region_forecast_ids(env, region);
    region_ids.push_back(forecast_id.clone());
    env.storage()
        .persistent()
        .set(&StorageKey::RegionForecasts(region.clone()), &region_ids);
}
