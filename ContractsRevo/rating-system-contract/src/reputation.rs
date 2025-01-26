use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

use crate::history;

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct ReputationRecord {
    pub score: u32,
    pub timestamp: u64,
}

// Calculates the reputation score based on the weighted rating
pub fn reputation_score_calculate(env: Env, seller: Address) -> u32 {
    // fetch the weighted rating for the seller
    let x = crate::rating::calculate_weighted_rating(env, seller);

    // Determine the reputation score based on the rating range
    match x {
        x if x <= 1.0 => 1,
        x if x <= 2.0 => 2,
        x if x <= 3.0 => 3,
        x if x <= 4.0 => 4,
        _ => 5,
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
    env.storage().instance().set(&seller, &reputation_history);

    env.events().publish(
        (
            Symbol::new(&env, "added_reputation_score_in_history"),
            seller.clone(),
        ),
        new_record,
    );
}
