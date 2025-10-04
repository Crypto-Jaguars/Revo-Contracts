use soroban_sdk::{contracterror, Address, Env, Map, String, Symbol};

use crate::token::{DataKey, Minters};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminError {
    Unauthorized = 1,
    AlreadyMinter = 2,
    NotMinter = 3,
    AlreadyPaused = 4,
    NotPaused = 5,
    NotInitialized = 6,
}

/// Get the admin address
pub fn get_admin(env: Env) -> Result<Address, AdminError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(AdminError::NotInitialized)
}

/// Check if an address is a minter
pub fn is_minter(env: Env, address: Address) -> bool {
    let minters = env
        .storage()
        .persistent()
        .get::<_, Minters>(&DataKey::Minters)
        .unwrap_or(Map::new(&env));

    minters.get(address).unwrap_or(false)
}

/// Add a new minter (admin only)
pub fn add_minter(env: Env, admin: Address, minter: Address) -> Result<(), AdminError> {
    admin.require_auth();

    // Verify admin
    let stored_admin = get_admin(env.clone())?;
    if admin != stored_admin {
        return Err(AdminError::Unauthorized);
    }

    // Get current minters
    let mut minters = env
        .storage()
        .persistent()
        .get::<_, Minters>(&DataKey::Minters)
        .unwrap_or(Map::new(&env));

    // Check if already a minter
    if minters.get(minter.clone()).unwrap_or(false) {
        return Err(AdminError::AlreadyMinter);
    }

    // Add new minter
    minters.set(minter.clone(), true);
    env.storage().persistent().set(&DataKey::Minters, &minters);

    // Emit event
    env.events()
        .publish((Symbol::new(&env, "add_minter"), admin, minter), ());

    Ok(())
}

/// Remove a minter (admin only)
pub fn remove_minter(env: Env, admin: Address, minter: Address) -> Result<(), AdminError> {
    admin.require_auth();

    // Verify admin
    let stored_admin = get_admin(env.clone())?;
    if admin != stored_admin {
        return Err(AdminError::Unauthorized);
    }

    // Get current minters
    let mut minters = env
        .storage()
        .persistent()
        .get::<_, Minters>(&DataKey::Minters)
        .unwrap_or(Map::new(&env));

    // Check if not a minter
    if !minters.get(minter.clone()).unwrap_or(false) {
        return Err(AdminError::NotMinter);
    }

    // Remove minter
    minters.remove(minter.clone());
    env.storage().persistent().set(&DataKey::Minters, &minters);

    // Emit event
    env.events()
        .publish((Symbol::new(&env, "remove_minter"), admin, minter), ());

    Ok(())
}

/// Pause token transfers (admin only)
pub fn pause(env: Env, admin: Address) -> Result<(), AdminError> {
    admin.require_auth();

    // Verify admin
    let stored_admin = get_admin(env.clone())?;
    if admin != stored_admin {
        return Err(AdminError::Unauthorized);
    }

    // Check if already paused
    let is_paused = env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Paused)
        .unwrap_or(false);

    if is_paused {
        return Err(AdminError::AlreadyPaused);
    }

    // Set paused state
    env.storage().instance().set(&DataKey::Paused, &true);

    // Emit event
    env.events()
        .publish((Symbol::new(&env, "pause"), admin), ());

    Ok(())
}

/// Unpause token transfers (admin only)
pub fn unpause(env: Env, admin: Address) -> Result<(), AdminError> {
    admin.require_auth();

    // Verify admin
    let stored_admin = get_admin(env.clone())?;
    if admin != stored_admin {
        return Err(AdminError::Unauthorized);
    }

    // Check if not paused
    let is_paused = env
        .storage()
        .instance()
        .get::<_, bool>(&DataKey::Paused)
        .unwrap_or(false);

    if !is_paused {
        return Err(AdminError::NotPaused);
    }

    // Set paused state
    env.storage().instance().set(&DataKey::Paused, &false);

    // Emit event
    env.events()
        .publish((Symbol::new(&env, "unpause"), admin), ());

    Ok(())
}

/// Check if token transfers are paused
pub fn is_paused(env: Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

/// Validate farmer address for agricultural operations
/// This can be extended to check if address is registered as a farmer
pub fn validate_farmer_address(env: &Env, farmer: &Address) -> Result<(), AdminError> {
    // For now, just ensure the address is valid
    // In future, this could check against a farmer registry
    if farmer == &Address::from_string(&String::from_str(env, "")) {
        return Err(AdminError::Unauthorized);
    }
    Ok(())
}

/// Calculate token amount based on agricultural metrics
/// This helper can be used to standardize reward calculations
pub fn calculate_reward_amount(
    base_amount: i128,
    quality_multiplier: u32,
    quantity_multiplier: u32,
) -> i128 {
    let quality_factor = quality_multiplier.max(100); // minimum 100% (1.0x)
    let quantity_factor = quantity_multiplier.max(100);

    // Calculate reward with multipliers (basis points)
    (base_amount * quality_factor as i128 * quantity_factor as i128) / 10000
}
