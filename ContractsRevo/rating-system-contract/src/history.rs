use crate::DataKey;
use crate::{datatypes::Error, rating::Rating, reputation::ReputationRecord};
use soroban_sdk::{Address, Env, Vec};

// Retrieves the rating history for a given seller
pub fn _get_rating_history(env: Env, seller: &Address) -> Result<Vec<Rating>, Error> {
    let key = DataKey::ReputationHistory(seller.clone());
    match env.storage().instance().get(&key) {
        Some(ratings) => Ok(ratings),
        None => Err(Error::RatingHistoryNotFound),
    }
}

// Retrieves the reputation history for a given seller
pub fn get_reputation_history(env: Env, seller: Address) -> Result<Vec<ReputationRecord>, Error> {
    let key = DataKey::ReputationHistory(seller.clone());
    match env.storage().instance().get(&key) {
        Some(reputation_record) => Ok(reputation_record),
        None => Err(Error::ReputationHistoryNotFound),
    }
}
