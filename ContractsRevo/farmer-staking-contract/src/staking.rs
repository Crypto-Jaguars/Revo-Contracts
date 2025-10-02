use soroban_sdk::{
    contracttype, Address, BytesN, Env, Symbol, Vec,
    contracterror,
};

use crate::pool::{get_pool_info, update_total_staked, update_epoch, is_pool_paused};
use crate::rewards::{calculate_pending_rewards, update_reward_debt};
use crate::utils::{transfer_from_user, transfer_to_user};

/// Errors that can occur in staking operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StakeError {
    PoolNotFound = 1,
    InsufficientAmount = 2,
    BelowMinimumStake = 3,
    ExceedsMaxLockPeriod = 4,
    StakeLocked = 5,
    NoStakeFound = 6,
    InsufficientStake = 7,
    Unauthorized = 8,
    PoolPaused = 9,
    TransferFailed = 10,
    PoolError = 11,
}

/// Individual stake information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stake {
    pub farmer_id: Address,
    pub pool_id: BytesN<32>,
    pub amount: i128,
    pub stake_time: u64,
    pub lock_period: u64,
    pub unlock_time: u64,
    pub reward_debt: i128,
}

/// Storage keys for stake data
#[contracttype]
#[derive(Clone)]
pub enum StakeStorageKey {
    Stake(Address, BytesN<32>),
    StakerList(BytesN<32>),
}

/// Stake tokens into a pool
pub fn stake(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
    amount: i128,
    lock_period: u64,
) -> Result<(), StakeError> {
    farmer.require_auth();

    // Get pool info
    let pool = get_pool_info(env.clone(), pool_id.clone())
        .map_err(|_| StakeError::PoolNotFound)?;

    // Check if pool is paused
    if is_pool_paused(env.clone(), pool_id.clone())
        .map_err(|_| StakeError::PoolError)?
    {
        return Err(StakeError::PoolPaused);
    }

    // Validate stake amount
    if amount <= 0 {
        return Err(StakeError::InsufficientAmount);
    }
    if amount < pool.min_stake_amount {
        return Err(StakeError::BelowMinimumStake);
    }

    // Validate lock period
    if lock_period > pool.max_lock_period {
        return Err(StakeError::ExceedsMaxLockPeriod);
    }

    let current_time = env.ledger().timestamp();
    let unlock_time = current_time.checked_add(lock_period).unwrap_or(u64::MAX);

    // Transfer tokens from farmer to contract
    transfer_from_user(
        env.clone(),
        pool.token_address.clone(),
        farmer.clone(),
        amount,
    ).map_err(|_| StakeError::TransferFailed)?;

    // Get or create stake
    let stake_key = StakeStorageKey::Stake(farmer.clone(), pool_id.clone());
    let mut stake: Stake = env
        .storage()
        .persistent()
        .get(&stake_key)
        .unwrap_or(Stake {
            farmer_id: farmer.clone(),
            pool_id: pool_id.clone(),
            amount: 0,
            stake_time: current_time,
            lock_period,
            unlock_time,
            reward_debt: 0,
        });

    // Calculate and claim any pending rewards before updating stake
    if stake.amount > 0 {
        let pending_rewards = calculate_pending_rewards(
            env.clone(),
            stake.clone(),
            pool.clone(),
        ).unwrap_or(0);

        if pending_rewards > 0 {
            transfer_to_user(
                env.clone(),
                pool.token_address.clone(),
                farmer.clone(),
                pending_rewards,
            ).map_err(|_| StakeError::TransferFailed)?;
        }
    }

    // Update stake
    stake.amount = stake.amount.checked_add(amount).unwrap_or(stake.amount);
    stake.stake_time = current_time;

    // Only update lock if new lock is longer
    if unlock_time > stake.unlock_time {
        stake.lock_period = lock_period;
        stake.unlock_time = unlock_time;
    }

    // Update reward debt
    stake.reward_debt = update_reward_debt(stake.amount, pool.clone());

    // Store updated stake
    env.storage()
        .persistent()
        .set(&stake_key, &stake);

    // Add to staker list if new staker
    let staker_list_key = StakeStorageKey::StakerList(pool_id.clone());
    let mut staker_list: Vec<Address> = env
        .storage()
        .persistent()
        .get(&staker_list_key)
        .unwrap_or(Vec::new(&env));

    let mut is_new_staker = true;
    for staker in staker_list.iter() {
        if staker == farmer {
            is_new_staker = false;
            break;
        }
    }

    if is_new_staker {
        staker_list.push_back(farmer.clone());
        env.storage()
            .persistent()
            .set(&staker_list_key, &staker_list);
    }

    // Update pool total staked
    update_total_staked(env.clone(), pool_id.clone(), amount)
        .map_err(|_| StakeError::PoolError)?;

    // Update epoch
    update_epoch(env.clone(), pool_id.clone())
        .map_err(|_| StakeError::PoolError)?;

    // Log event
    env.events().publish(
        (Symbol::new(&env, "staked"), farmer),
        (pool_id, amount, lock_period),
    );

    Ok(())
}

/// Unstake tokens from a pool
pub fn unstake(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
    amount: i128,
) -> Result<(), StakeError> {
    farmer.require_auth();

    // Get pool info
    let pool = get_pool_info(env.clone(), pool_id.clone())
        .map_err(|_| StakeError::PoolNotFound)?;

    // Get stake
    let stake_key = StakeStorageKey::Stake(farmer.clone(), pool_id.clone());
    let mut stake: Stake = env
        .storage()
        .persistent()
        .get(&stake_key)
        .ok_or(StakeError::NoStakeFound)?;

    // Validate unstake amount
    if amount <= 0 || amount > stake.amount {
        return Err(StakeError::InsufficientStake);
    }

    // Check if stake is locked
    let current_time = env.ledger().timestamp();
    if current_time < stake.unlock_time {
        return Err(StakeError::StakeLocked);
    }

    // Calculate and transfer pending rewards
    let pending_rewards = calculate_pending_rewards(
        env.clone(),
        stake.clone(),
        pool.clone(),
    ).unwrap_or(0);

    let total_transfer = amount.checked_add(pending_rewards).unwrap_or(amount);

    transfer_to_user(
        env.clone(),
        pool.token_address.clone(),
        farmer.clone(),
        total_transfer,
    ).map_err(|_| StakeError::TransferFailed)?;

    // Update stake
    stake.amount = stake.amount.checked_sub(amount).unwrap_or(0);

    if stake.amount == 0 {
        // Remove stake if fully unstaked
        env.storage().persistent().remove(&stake_key);
    } else {
        // Update reward debt for remaining stake
        stake.reward_debt = update_reward_debt(stake.amount, pool.clone());
        env.storage().persistent().set(&stake_key, &stake);
    }

    // Update pool total staked
    update_total_staked(env.clone(), pool_id.clone(), -amount)
        .map_err(|_| StakeError::PoolError)?;

    // Update epoch
    update_epoch(env.clone(), pool_id.clone())
        .map_err(|_| StakeError::PoolError)?;

    // Log event
    env.events().publish(
        (Symbol::new(&env, "unstaked"), farmer),
        (pool_id, amount, pending_rewards),
    );

    Ok(())
}

/// Emergency unstake with penalty
pub fn emergency_unstake(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
    amount: i128,
) -> Result<i128, StakeError> {
    farmer.require_auth();

    // Get pool info
    let pool = get_pool_info(env.clone(), pool_id.clone())
        .map_err(|_| StakeError::PoolNotFound)?;

    // Get stake
    let stake_key = StakeStorageKey::Stake(farmer.clone(), pool_id.clone());
    let mut stake: Stake = env
        .storage()
        .persistent()
        .get(&stake_key)
        .ok_or(StakeError::NoStakeFound)?;

    // Validate unstake amount
    if amount <= 0 || amount > stake.amount {
        return Err(StakeError::InsufficientStake);
    }

    // Calculate penalty (10% for early unstaking)
    let penalty_rate = 10i128;
    let penalty = amount.checked_mul(penalty_rate).unwrap_or(0) / 100;
    let amount_after_penalty = amount.checked_sub(penalty).unwrap_or(0);

    // Transfer amount after penalty
    transfer_to_user(
        env.clone(),
        pool.token_address.clone(),
        farmer.clone(),
        amount_after_penalty,
    ).map_err(|_| StakeError::TransferFailed)?;

    // Penalty stays in contract as additional rewards for other stakers

    // Update stake
    stake.amount = stake.amount.checked_sub(amount).unwrap_or(0);

    if stake.amount == 0 {
        env.storage().persistent().remove(&stake_key);
    } else {
        stake.reward_debt = update_reward_debt(stake.amount, pool.clone());
        env.storage().persistent().set(&stake_key, &stake);
    }

    // Update pool total staked
    update_total_staked(env.clone(), pool_id.clone(), -amount)
        .map_err(|_| StakeError::PoolError)?;

    // Update epoch
    update_epoch(env.clone(), pool_id.clone())
        .map_err(|_| StakeError::PoolError)?;

    // Log event
    env.events().publish(
        (Symbol::new(&env, "emergency_unstaked"), farmer),
        (pool_id, amount, penalty),
    );

    Ok(amount_after_penalty)
}

/// Get stake information and pending rewards
pub fn get_stake_info(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
) -> Result<(Stake, i128), StakeError> {
    let stake_key = StakeStorageKey::Stake(farmer, pool_id.clone());
    let stake: Stake = env
        .storage()
        .persistent()
        .get(&stake_key)
        .ok_or(StakeError::NoStakeFound)?;

    let pool = get_pool_info(env.clone(), pool_id)
        .map_err(|_| StakeError::PoolNotFound)?;

    let pending_rewards = calculate_pending_rewards(env, stake.clone(), pool).unwrap_or(0);

    Ok((stake, pending_rewards))
}

/// Get all stakers in a pool
pub fn get_stakers(env: Env, pool_id: BytesN<32>) -> Vec<Address> {
    let staker_list_key = StakeStorageKey::StakerList(pool_id);
    env.storage()
        .persistent()
        .get(&staker_list_key)
        .unwrap_or(Vec::new(&env))
}
