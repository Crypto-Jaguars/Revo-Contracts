use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

use crate::{history, DataKey};

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct ReputationRecord {
    pub score: u32,
    pub timestamp: u64,
}

// Calculates the reputation score based on the weighted rating
pub fn reputation_score_calculate(env: Env, seller: Address) -> u32 {
    // fetch the weighted rating for the seller (scaled by 100)
    let weighted_rating_scaled = crate::rating::calculate_weighted_rating(env, seller);

    // Determine the reputation score based on the rating range
    // Since the rating is scaled by 100, we compare against scaled values
    match weighted_rating_scaled {
        x if x <= 100 => 1,  // <= 1.0
        x if x <= 200 => 2,  // <= 2.0
        x if x <= 300 => 3,  // <= 3.0
        x if x <= 400 => 4,  // <= 4.0
        _ => 5,              // > 4.0
    }
}

// Adds a new reputation score record to the seller's history
pub fn add_reputation_score_history(env: Env, seller: Address, score: u32) {
    // Get current ledger timestamp
    let timestamp = env.ledger().timestamp();

    // Retrieve existing reputation history or initialize a new vector
    let mut reputation_history: Vec<ReputationRecord> =
        match history::get_reputation_history(env.clone(), seller.clone()) {
            Ok(history) => history,
            Err(_) => Vec::new(&env),
        };

    // Create a new reputation record
    let new_record = ReputationRecord { score, timestamp };

    // Add the new record to the history
    reputation_history.push_back(new_record.clone());

    // Update the seller's reputation history in storage
    let key = DataKey::ReputationHistory(seller.clone());
    env.storage().instance().set(&key, &reputation_history);

    env.events().publish(
        (Symbol::new(&env, "added_score_in_history"), seller.clone()),
        new_record,
    );
}
