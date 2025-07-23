use crate::{DataKey, Reputation, TradeError};
use soroban_sdk::{Address, Env};

pub fn update_reputation_after_trade(
    env: &Env,
    cooperative_id: &Address,
    trade_successful: bool,
) -> Result<(), TradeError> {
    // Get existing reputation or create new one
    let mut reputation: Reputation = env
        .storage()
        .persistent()
        .get(&DataKey::Reputation(cooperative_id.clone()))
        .unwrap_or(Reputation {
            cooperative_id: cooperative_id.clone(),
            successful_trades: 0,
            rating: 5, // Start with max rating
        });

    // Update trade counts (simplified - just track successful trades)
    if trade_successful {
        reputation.successful_trades += 1;
    }

    // Simple rating calculation based on successful trades
    reputation.rating = if reputation.successful_trades >= 10 {
        5
    } else if reputation.successful_trades >= 5 {
        4
    } else if reputation.successful_trades >= 2 {
        3
    } else if reputation.successful_trades >= 1 {
        2
    } else {
        1
    };

    // Store updated reputation
    env.storage()
        .persistent()
        .set(&DataKey::Reputation(cooperative_id.clone()), &reputation);

    Ok(())
}
