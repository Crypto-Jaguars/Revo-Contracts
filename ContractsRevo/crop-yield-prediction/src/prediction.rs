use crate::types::{Crop, CropYieldError, DataKey, DataSource, YieldPrediction};
use crate::utils;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

#[contract]
pub struct CropYieldPredictionContract;

#[contractimpl]
impl CropYieldPredictionContract {
    /// Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), CropYieldError> {
        env.storage().instance().set(&DataKey::ADMIN, &admin);
        Ok(())
    }

    /// Register a new crop with historical yield data
    pub fn register_crop(
        env: Env,
        crop_id: BytesN<32>,
        name: String,
        historical_yields: Vec<i128>,
    ) -> Result<BytesN<32>, CropYieldError> {
        let admin: Address = match env.storage().instance().get(&DataKey::ADMIN) {
            Some(admin) => admin,
            None => return Err(CropYieldError::ContractNotInitialized),
        };
        admin.require_auth();

        if name.len() == 0 || historical_yields.is_empty() {
            return Err(CropYieldError::InvalidInput);
        }

        let crop = Crop {
            crop_id: crop_id.clone(),
            name,
            historical_yields,
        };

        env.storage().persistent().set(&crop_id, &crop);

        // Store crop ID in crops list
        let mut crops: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::CROPS)
            .unwrap_or(Vec::new(&env));
        crops.push_back(crop_id.clone());
        env.storage().persistent().set(&DataKey::CROPS, &crops);

        Ok(crop_id)
    }

    /// Generate a yield prediction based on oracle data
    pub fn generate_prediction(
        env: Env,
        crop_id: BytesN<32>,
        region: String,
        data_source: DataSource,
    ) -> Result<BytesN<32>, CropYieldError> {
        if region.len() == 0 {
            return Err(CropYieldError::InvalidInput);
        }

        let crop: Crop = match env.storage().persistent().get(&crop_id) {
            Some(crop) => crop,
            None => return Err(CropYieldError::CropNotFound),
        };

        let prediction_id = utils::generate_prediction_id(&env, &crop_id);
        let data_hash = utils::hash_data_source(&env, &data_source);
        let predicted_yield = utils::calculate_yield_prediction(&crop, &data_source);

        let prediction = YieldPrediction {
            prediction_id: prediction_id.clone(),
            crop_id,
            region,
            predicted_yield,
            data_hash,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&prediction_id, &prediction);

        let mut predictions: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::PREDICTIONS)
            .unwrap_or(Vec::new(&env));
        predictions.push_back(prediction_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::PREDICTIONS, &predictions);

        Ok(prediction_id)
    }

    /// Get a specific yield prediction
    pub fn get_prediction(
        env: Env,
        prediction_id: BytesN<32>,
    ) -> Result<YieldPrediction, CropYieldError> {
        match env.storage().persistent().get(&prediction_id) {
            Some(prediction) => Ok(prediction),
            None => Err(CropYieldError::PredictionNotFound),
        }
    }

    /// List all predictions for a specific crop
    pub fn list_predictions_by_crop(
        env: Env,
        crop_id: BytesN<32>,
    ) -> Result<Vec<YieldPrediction>, CropYieldError> {
        let predictions_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::PREDICTIONS)
            .unwrap_or(Vec::new(&env));

        let mut crop_predictions = Vec::new(&env);

        for pred_id in predictions_ids.iter() {
            if let Ok(prediction) = Self::get_prediction(env.clone(), pred_id) {
                if prediction.crop_id == crop_id {
                    crop_predictions.push_back(prediction);
                }
            }
        }

        Ok(crop_predictions)
    }

    /// List all predictions for a specific region
    pub fn list_predictions_by_region(
        env: Env,
        region: String,
    ) -> Result<Vec<YieldPrediction>, CropYieldError> {
        if region.len() == 0 {
            return Err(CropYieldError::InvalidInput);
        }

        let predictions_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::PREDICTIONS)
            .unwrap_or(Vec::new(&env));

        let mut region_predictions = Vec::new(&env);

        for pred_id in predictions_ids.iter() {
            if let Ok(prediction) = Self::get_prediction(env.clone(), pred_id) {
                if prediction.region == region {
                    region_predictions.push_back(prediction);
                }
            }
        }

        Ok(region_predictions)
    }

    /// Update data source (admin only)
    pub fn update_data_source(
        env: Env,
        prediction_id: BytesN<32>,
        new_data_source: DataSource,
    ) -> Result<BytesN<32>, CropYieldError> {
        let admin: Address = match env.storage().instance().get(&DataKey::ADMIN) {
            Some(admin) => admin,
            None => return Err(CropYieldError::ContractNotInitialized),
        };
        admin.require_auth();

        let mut prediction: YieldPrediction = match env.storage().persistent().get(&prediction_id) {
            Some(prediction) => prediction,
            None => return Err(CropYieldError::PredictionNotFound),
        };

        // Update data hash
        prediction.data_hash = utils::hash_data_source(&env, &new_data_source);
        prediction.timestamp = env.ledger().timestamp();

        // Recalculate prediction
        let crop: Crop = match env.storage().persistent().get(&prediction.crop_id) {
            Some(crop) => crop,
            None => return Err(CropYieldError::CropNotFound),
        };
        prediction.predicted_yield = utils::calculate_yield_prediction(&crop, &new_data_source);

        env.storage().persistent().set(&prediction_id, &prediction);

        Ok(prediction_id)
    }

    /// Get crop information
    pub fn get_crop(env: Env, crop_id: BytesN<32>) -> Result<Crop, CropYieldError> {
        match env.storage().persistent().get(&crop_id) {
            Some(crop) => Ok(crop),
            None => Err(CropYieldError::CropNotFound),
        }
    }
}
