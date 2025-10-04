use soroban_sdk::{contracterror, Address, BytesN, Env, Symbol};

use crate::pool::{get_pool_info, update_epoch, RewardPool};
use crate::staking::{Stake, StakeStorageKey};
use crate::utils::transfer_to_user;

/// Errors that can occur in reward operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RewardError {
    PoolNotFound = 1,
    StakeNotFound = 2,
    NoRewardsToClaim = 3,
    TransferFailed = 4,
    CalculationError = 5,
}

/// Calculate pending rewards for a stake
pub fn calculate_pending_rewards(
    env: Env,
    stake: Stake,
    pool: RewardPool,
) -> Result<i128, RewardError> {
    if stake.amount == 0 {
        return Ok(0);
    }

    let current_time = env.ledger().timestamp();
    let time_staked = current_time.checked_sub(stake.stake_time).unwrap_or(0);

    // Calculate base rewards
    // Reward formula: (staked_amount * reward_rate * time_staked) / (total_staked * epoch_duration)
    let epoch_duration = 86400u64; // 1 day in seconds

    if pool.total_staked == 0 {
        return Ok(0);
    }

    // Calculate user's share of the pool
    let user_share = stake.amount * 1_000_000 / pool.total_staked; // Scale for precision

    // Calculate epochs passed since stake
    let epochs_passed = time_staked / epoch_duration;

    // Calculate rewards
    let base_rewards = (pool.reward_rate * user_share * epochs_passed as i128) / 1_000_000;

    // Apply lock period multiplier for bonus rewards
    // Longer lock periods get higher rewards
    let lock_multiplier = calculate_lock_multiplier(stake.lock_period);
    let rewards_with_multiplier = (base_rewards * lock_multiplier) / 100;

    // Subtract reward debt (already claimed rewards)
    let pending_rewards = rewards_with_multiplier
        .checked_sub(stake.reward_debt)
        .unwrap_or(0);

    Ok(pending_rewards.max(0))
}

/// Calculate lock period multiplier for bonus rewards
/// Returns multiplier as percentage (100 = no bonus, 150 = 50% bonus)
pub fn calculate_lock_multiplier(lock_period: u64) -> i128 {
    // Base multiplier
    let base = 100i128;

    // Bonus tiers
    let one_week = 604800u64;
    let one_month = 2592000u64;
    let three_months = 7776000u64;
    let six_months = 15552000u64;
    let one_year = 31536000u64;

    if lock_period == 0 {
        base // No lock = no bonus
    } else if lock_period < one_week {
        105 // < 1 week = 5% bonus
    } else if lock_period < one_month {
        110 // < 1 month = 10% bonus
    } else if lock_period < three_months {
        120 // < 3 months = 20% bonus
    } else if lock_period < six_months {
        135 // < 6 months = 35% bonus
    } else if lock_period < one_year {
        150 // < 1 year = 50% bonus
    } else {
        175 // >= 1 year = 75% bonus
    }
}

/// Update reward debt after claiming or changing stake
pub fn update_reward_debt(stake_amount: i128, pool: RewardPool) -> i128 {
    // Reward debt tracks the rewards already accounted for
    // When staking more or claiming, we reset the baseline
    let current_accumulated_reward_per_share = if pool.total_staked > 0 {
        (pool.reward_rate * pool.current_epoch as i128 * 1_000_000) / pool.total_staked
    } else {
        0
    };

    (stake_amount * current_accumulated_reward_per_share) / 1_000_000
}

/// Claim pending rewards without unstaking
pub fn claim_rewards(env: Env, farmer: Address, pool_id: BytesN<32>) -> Result<i128, RewardError> {
    farmer.require_auth();

    // Update epoch before calculating rewards
    update_epoch(env.clone(), pool_id.clone()).map_err(|_| RewardError::CalculationError)?;

    // Get updated pool info
    let pool =
        get_pool_info(env.clone(), pool_id.clone()).map_err(|_| RewardError::PoolNotFound)?;

    // Get stake
    let stake_key = StakeStorageKey::Stake(farmer.clone(), pool_id.clone());
    let mut stake: Stake = env
        .storage()
        .persistent()
        .get(&stake_key)
        .ok_or(RewardError::StakeNotFound)?;

    // Calculate pending rewards
    let pending_rewards = calculate_pending_rewards(env.clone(), stake.clone(), pool.clone())?;

    if pending_rewards == 0 {
        return Err(RewardError::NoRewardsToClaim);
    }

    // Transfer rewards to farmer
    transfer_to_user(
        env.clone(),
        pool.token_address.clone(),
        farmer.clone(),
        pending_rewards,
    )
    .map_err(|_| RewardError::TransferFailed)?;

    // Update reward debt
    stake.reward_debt = stake
        .reward_debt
        .checked_add(pending_rewards)
        .unwrap_or(stake.reward_debt);

    env.storage().persistent().set(&stake_key, &stake);

    // Log event
    env.events().publish(
        (Symbol::new(&env, "rewards_claimed"), farmer),
        (pool_id, pending_rewards),
    );

    Ok(pending_rewards)
}

/// Compound rewards by restaking them
pub fn compound_rewards(
    env: Env,
    farmer: Address,
    pool_id: BytesN<32>,
) -> Result<i128, RewardError> {
    farmer.require_auth();

    // Update epoch
    update_epoch(env.clone(), pool_id.clone()).map_err(|_| RewardError::CalculationError)?;

    // Get updated pool info
    let pool =
        get_pool_info(env.clone(), pool_id.clone()).map_err(|_| RewardError::PoolNotFound)?;

    // Get stake
    let stake_key = StakeStorageKey::Stake(farmer.clone(), pool_id.clone());
    let mut stake: Stake = env
        .storage()
        .persistent()
        .get(&stake_key)
        .ok_or(RewardError::StakeNotFound)?;

    // Calculate pending rewards
    let pending_rewards = calculate_pending_rewards(env.clone(), stake.clone(), pool.clone())?;

    if pending_rewards == 0 {
        return Err(RewardError::NoRewardsToClaim);
    }

    // Add rewards to stake amount (compound)
    stake.amount = stake
        .amount
        .checked_add(pending_rewards)
        .unwrap_or(stake.amount);

    // Update reward debt
    stake.reward_debt = update_reward_debt(stake.amount, pool.clone());

    env.storage().persistent().set(&stake_key, &stake);

    // Update pool total staked (rewards are now staked)
    use crate::pool::update_total_staked;
    update_total_staked(env.clone(), pool_id.clone(), pending_rewards)
        .map_err(|_| RewardError::CalculationError)?;

    // Log event
    env.events().publish(
        (Symbol::new(&env, "rewards_compounded"), farmer),
        (pool_id, pending_rewards),
    );

    Ok(pending_rewards)
}

/// Calculate APR for a given lock period
/// Returns APR as basis points (10000 = 100%)
pub fn calculate_apr(env: Env, pool_id: BytesN<32>, lock_period: u64) -> Result<i128, RewardError> {
    let pool = get_pool_info(env, pool_id).map_err(|_| RewardError::PoolNotFound)?;

    if pool.total_staked == 0 {
        return Ok(0);
    }

    // Base APR = (reward_rate * 365 days) / total_staked
    let yearly_rewards = pool.reward_rate * 365;
    let base_apr = (yearly_rewards * 10000) / pool.total_staked;

    // Apply lock multiplier
    let multiplier = calculate_lock_multiplier(lock_period);
    let apr_with_bonus = (base_apr * multiplier) / 100;

    Ok(apr_with_bonus)
}

/// Get total rewards distributed from a pool
pub fn get_total_rewards_distributed(env: Env, pool_id: BytesN<32>) -> Result<i128, RewardError> {
    let pool = get_pool_info(env, pool_id).map_err(|_| RewardError::PoolNotFound)?;

    // Total rewards = reward_rate * epochs_passed
    let total_rewards = pool.reward_rate * pool.current_epoch as i128;

    Ok(total_rewards)
}
