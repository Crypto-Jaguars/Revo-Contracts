use soroban_sdk::{contracterror, Address, Env, Symbol};

use crate::token::{update_total_supply, DataKey};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BurnError {
    Unauthorized = 1,
    InvalidAmount = 2,
    InsufficientBalance = 3,
    Paused = 4,
}

/// Burn tokens from an address
/// Either the token holder themselves or an approved burner can burn tokens
pub fn burn_tokens(
    env: Env,
    burner: Address,
    from: Address,
    amount: i128,
) -> Result<(), BurnError> {
    // Either the owner or an authorized burner must approve
    if burner == from {
        from.require_auth();
    } else {
        burner.require_auth();
        // Check if burner has allowance to burn from this address
        let allowance = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Allowance(from.clone(), burner.clone()))
            .unwrap_or(0);

        if allowance < amount {
            return Err(BurnError::Unauthorized);
        }

        // Update allowance
        let new_allowance = allowance - amount;
        if new_allowance == 0 {
            env.storage()
                .persistent()
                .remove(&DataKey::Allowance(from.clone(), burner.clone()));
        } else {
            env.storage().persistent().set(
                &DataKey::Allowance(from.clone(), burner.clone()),
                &new_allowance,
            );
        }
    }

    // Check if the contract is paused
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Paused)
        .unwrap_or(false)
    {
        return Err(BurnError::Paused);
    }

    // Validate amount
    if amount <= 0 {
        return Err(BurnError::InvalidAmount);
    }

    // Get current balance
    let current_balance = env
        .storage()
        .persistent()
        .get::<_, i128>(&DataKey::Balance(from.clone()))
        .unwrap_or(0);

    // Check sufficient balance
    if current_balance < amount {
        return Err(BurnError::InsufficientBalance);
    }

    // Update balance
    let new_balance = current_balance - amount;
    if new_balance == 0 {
        env.storage()
            .persistent()
            .remove(&DataKey::Balance(from.clone()));
    } else {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_balance);
    }

    // Update total supply
    let current_supply = env
        .storage()
        .instance()
        .get::<_, i128>(&DataKey::TotalSupply)
        .unwrap_or(0);

    let new_supply = current_supply - amount;
    update_total_supply(&env, new_supply);

    // Emit burn event
    env.events().publish(
        (Symbol::new(&env, "burn"), burner, from.clone()),
        (amount, new_balance, new_supply),
    );

    Ok(())
}

/// Burn tokens for redemption purposes
/// This can be used when farmers redeem tokens for real-world value or services
pub fn burn_for_redemption(
    env: Env,
    farmer: Address,
    amount: i128,
    redemption_type: Symbol,
) -> Result<(), BurnError> {
    farmer.require_auth();

    // Check if the contract is paused
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Paused)
        .unwrap_or(false)
    {
        return Err(BurnError::Paused);
    }

    // Validate amount
    if amount <= 0 {
        return Err(BurnError::InvalidAmount);
    }

    // Get current balance
    let current_balance = env
        .storage()
        .persistent()
        .get::<_, i128>(&DataKey::Balance(farmer.clone()))
        .unwrap_or(0);

    // Check sufficient balance
    if current_balance < amount {
        return Err(BurnError::InsufficientBalance);
    }

    // Update balance
    let new_balance = current_balance - amount;
    if new_balance == 0 {
        env.storage()
            .persistent()
            .remove(&DataKey::Balance(farmer.clone()));
    } else {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(farmer.clone()), &new_balance);
    }

    // Update total supply
    let current_supply = env
        .storage()
        .instance()
        .get::<_, i128>(&DataKey::TotalSupply)
        .unwrap_or(0);

    let new_supply = current_supply - amount;
    update_total_supply(&env, new_supply);

    // Emit burn event
    env.events().publish(
        (Symbol::new(&env, "burn"), farmer.clone(), farmer.clone()),
        (amount, new_balance, new_supply),
    );

    // Emit redemption-specific event
    env.events().publish(
        (
            Symbol::new(&env, "redemption_burn"),
            farmer,
            redemption_type,
        ),
        amount,
    );

    Ok(())
}

/// Burn tokens as a penalty
/// This can be used by admin/governance for penalizing bad actors
pub fn burn_as_penalty(
    env: Env,
    admin: Address,
    from: Address,
    amount: i128,
    reason: Symbol,
) -> Result<(), BurnError> {
    admin.require_auth();

    // Verify admin
    let stored_admin = env
        .storage()
        .instance()
        .get::<_, Address>(&DataKey::Admin)
        .ok_or(BurnError::Unauthorized)?;

    if admin != stored_admin {
        return Err(BurnError::Unauthorized);
    }

    // Get current balance
    let current_balance = env
        .storage()
        .persistent()
        .get::<_, i128>(&DataKey::Balance(from.clone()))
        .unwrap_or(0);

    // Check sufficient balance
    if current_balance < amount {
        return Err(BurnError::InsufficientBalance);
    }

    // Update balance
    let new_balance = current_balance - amount;
    if new_balance == 0 {
        env.storage()
            .persistent()
            .remove(&DataKey::Balance(from.clone()));
    } else {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_balance);
    }

    // Update total supply
    let current_supply = env
        .storage()
        .instance()
        .get::<_, i128>(&DataKey::TotalSupply)
        .unwrap_or(0);

    let new_supply = current_supply - amount;
    update_total_supply(&env, new_supply);

    // Emit penalty burn event
    env.events().publish(
        (
            Symbol::new(&env, "penalty_burn"),
            admin,
            from.clone(),
            reason,
        ),
        amount,
    );

    Ok(())
}
