use soroban_sdk::{contracterror, Address, Env, Symbol};

use crate::{
    token::{update_total_supply, DataKey},
    utils::is_minter,
};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MintError {
    Unauthorized = 1,
    InvalidAmount = 2,
    Paused = 3,
}

/// Mint new tokens to a farmer's address
/// Only authorized minters can mint tokens
pub fn mint_tokens(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), MintError> {
    minter.require_auth();

    // Check if the minter is authorized
    if !is_minter(env.clone(), minter.clone()) {
        return Err(MintError::Unauthorized);
    }

    // Check if the contract is paused
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Paused)
        .unwrap_or(false)
    {
        return Err(MintError::Paused);
    }

    // Validate amount
    if amount <= 0 {
        return Err(MintError::InvalidAmount);
    }

    // Get current balance of the recipient
    let current_balance = env
        .storage()
        .persistent()
        .get::<_, i128>(&DataKey::Balance(to.clone()))
        .unwrap_or(0);

    // Update the balance
    let new_balance = current_balance + amount;
    env.storage()
        .persistent()
        .set(&DataKey::Balance(to.clone()), &new_balance);

    // Update total supply
    let current_supply = env
        .storage()
        .instance()
        .get::<_, i128>(&DataKey::TotalSupply)
        .unwrap_or(0);

    let new_supply = current_supply + amount;
    update_total_supply(&env, new_supply);

    // Emit mint event with agricultural context
    env.events().publish(
        (Symbol::new(&env, "mint"), minter, to.clone()),
        (amount, new_balance, new_supply),
    );

    Ok(())
}

/// Mint tokens for agricultural milestones
/// This function can be called by authorized systems when farmers achieve certain milestones
pub fn mint_for_milestone(
    env: Env,
    minter: Address,
    farmer: Address,
    milestone_type: Symbol,
    amount: i128,
) -> Result<(), MintError> {
    minter.require_auth();

    // Check if the minter is authorized
    if !is_minter(env.clone(), minter.clone()) {
        return Err(MintError::Unauthorized);
    }

    // Check if the contract is paused
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Paused)
        .unwrap_or(false)
    {
        return Err(MintError::Paused);
    }

    // Validate amount
    if amount <= 0 {
        return Err(MintError::InvalidAmount);
    }

    // Get current balance of the recipient
    let current_balance = env
        .storage()
        .persistent()
        .get::<_, i128>(&DataKey::Balance(farmer.clone()))
        .unwrap_or(0);

    // Update the balance
    let new_balance = current_balance + amount;
    env.storage()
        .persistent()
        .set(&DataKey::Balance(farmer.clone()), &new_balance);

    // Update total supply
    let current_supply = env
        .storage()
        .instance()
        .get::<_, i128>(&DataKey::TotalSupply)
        .unwrap_or(0);

    let new_supply = current_supply + amount;
    update_total_supply(&env, new_supply);

    // Emit mint event
    env.events().publish(
        (Symbol::new(&env, "mint"), minter, farmer.clone()),
        (amount, new_balance, new_supply),
    );

    // Emit milestone-specific event
    env.events().publish(
        (Symbol::new(&env, "milestone_mint"), farmer, milestone_type),
        amount,
    );

    Ok(())
}

/// Batch mint tokens to multiple farmers
/// Useful for distributing rewards after harvest seasons
pub fn batch_mint(
    env: Env,
    minter: Address,
    recipients: soroban_sdk::Vec<(Address, i128)>,
) -> Result<(), MintError> {
    minter.require_auth();

    // Check if the minter is authorized
    if !is_minter(env.clone(), minter.clone()) {
        return Err(MintError::Unauthorized);
    }

    // Check if the contract is paused
    if env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Paused)
        .unwrap_or(false)
    {
        return Err(MintError::Paused);
    }

    let mut total_minted = 0i128;

    // Process each recipient
    for (recipient, amount) in recipients.iter() {
        if amount <= 0 {
            return Err(MintError::InvalidAmount);
        }

        // Get current balance
        let current_balance = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Balance(recipient.clone()))
            .unwrap_or(0);

        // Update balance
        let new_balance = current_balance + amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(recipient.clone()), &new_balance);

        total_minted += amount;

        // Emit individual mint event
        env.events().publish(
            (Symbol::new(&env, "mint"), minter.clone(), recipient.clone()),
            amount,
        );
    }

    // Update total supply
    let current_supply = env
        .storage()
        .instance()
        .get::<_, i128>(&DataKey::TotalSupply)
        .unwrap_or(0);

    let new_supply = current_supply + total_minted;
    update_total_supply(&env, new_supply);

    // Emit batch mint event
    env.events()
        .publish((Symbol::new(&env, "batch_mint"), minter), total_minted);

    Ok(())
}
