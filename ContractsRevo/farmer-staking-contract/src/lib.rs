#![no_std]

mod pool;
mod rewards;
mod staking;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

pub use pool::{PoolError, RewardPool};
pub use rewards::RewardError;
pub use staking::{Stake, StakeError};
pub use utils::ValidationError;

/// Main contract for farmer staking functionality
#[contract]
pub struct FarmerStakingContract;

#[contractimpl]
impl FarmerStakingContract {
    /// Initialize a staking pool with reward settings
    ///
    /// # Arguments
    /// * `admin` - Address that will manage the pool
    /// * `token_address` - Address of the farmer token contract
    /// * `reward_rate` - Rewards per epoch (in token units)
    /// * `min_stake_amount` - Minimum amount required to stake
    /// * `max_lock_period` - Maximum lock period in seconds
    ///
    /// # Returns
    /// * `BytesN<32>` - Unique pool identifier
    pub fn initialize_pool(
        env: Env,
        admin: Address,
        token_address: Address,
        reward_rate: i128,
        min_stake_amount: i128,
        max_lock_period: u64,
    ) -> Result<BytesN<32>, PoolError> {
        pool::initialize_pool(
            env,
            admin,
            token_address,
            reward_rate,
            min_stake_amount,
            max_lock_period,
        )
    }

    /// Stake farmer tokens with an optional lock period
    ///
    /// # Arguments
    /// * `farmer` - Address of the farmer staking tokens
    /// * `pool_id` - Pool to stake into
    /// * `amount` - Amount of tokens to stake
    /// * `lock_period` - Duration in seconds to lock tokens (0 for no lock)
    ///
    /// # Returns
    /// * `Result<(), StakeError>`
    pub fn stake(
        env: Env,
        farmer: Address,
        pool_id: BytesN<32>,
        amount: i128,
        lock_period: u64,
    ) -> Result<(), StakeError> {
        staking::stake(env, farmer, pool_id, amount, lock_period)
    }

    /// Unstake tokens and claim accumulated rewards after lock period
    ///
    /// # Arguments
    /// * `farmer` - Address of the farmer unstaking tokens
    /// * `pool_id` - Pool to unstake from
    /// * `amount` - Amount of tokens to unstake
    ///
    /// # Returns
    /// * `Result<(), StakeError>`
    pub fn unstake(
        env: Env,
        farmer: Address,
        pool_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), StakeError> {
        staking::unstake(env, farmer, pool_id, amount)
    }

    /// Claim pending rewards without unstaking
    ///
    /// # Arguments
    /// * `farmer` - Address claiming rewards
    /// * `pool_id` - Pool to claim rewards from
    ///
    /// # Returns
    /// * `Result<i128, RewardError>` - Amount of rewards claimed
    pub fn claim_rewards(
        env: Env,
        farmer: Address,
        pool_id: BytesN<32>,
    ) -> Result<i128, RewardError> {
        rewards::claim_rewards(env, farmer, pool_id)
    }

    /// Query stake details and pending rewards for a farmer
    ///
    /// # Arguments
    /// * `farmer` - Address to query
    /// * `pool_id` - Pool to query from
    ///
    /// # Returns
    /// * `Result<(Stake, i128), StakeError>` - Stake info and pending rewards
    pub fn get_stake_info(
        env: Env,
        farmer: Address,
        pool_id: BytesN<32>,
    ) -> Result<(Stake, i128), StakeError> {
        staking::get_stake_info(env, farmer, pool_id)
    }

    /// Get pool information
    ///
    /// # Arguments
    /// * `pool_id` - Pool to query
    ///
    /// # Returns
    /// * `Result<RewardPool, PoolError>`
    pub fn get_pool_info(env: Env, pool_id: BytesN<32>) -> Result<RewardPool, PoolError> {
        pool::get_pool_info(env, pool_id)
    }

    /// Get all active pool IDs
    ///
    /// # Returns
    /// * `Vec<BytesN<32>>` - List of all pool IDs
    pub fn get_all_pools(env: Env) -> Vec<BytesN<32>> {
        pool::get_all_pools(env)
    }

    /// Update pool reward rate (admin only)
    ///
    /// # Arguments
    /// * `admin` - Address of the pool admin
    /// * `pool_id` - Pool to update
    /// * `new_reward_rate` - New reward rate per epoch
    ///
    /// # Returns
    /// * `Result<(), PoolError>`
    pub fn update_reward_rate(
        env: Env,
        admin: Address,
        pool_id: BytesN<32>,
        new_reward_rate: i128,
    ) -> Result<(), PoolError> {
        pool::update_reward_rate(env, admin, pool_id, new_reward_rate)
    }

    /// Compound rewards by restaking them
    ///
    /// # Arguments
    /// * `farmer` - Address compounding rewards
    /// * `pool_id` - Pool to compound in
    ///
    /// # Returns
    /// * `Result<i128, RewardError>` - Amount of rewards compounded
    pub fn compound_rewards(
        env: Env,
        farmer: Address,
        pool_id: BytesN<32>,
    ) -> Result<i128, RewardError> {
        rewards::compound_rewards(env, farmer, pool_id)
    }

    /// Get total value locked in a pool
    ///
    /// # Arguments
    /// * `pool_id` - Pool to query
    ///
    /// # Returns
    /// * `Result<i128, PoolError>` - Total amount staked
    pub fn get_total_staked(env: Env, pool_id: BytesN<32>) -> Result<i128, PoolError> {
        pool::get_total_staked(env, pool_id)
    }

    /// Emergency unstake with penalty (applies slashing)
    ///
    /// # Arguments
    /// * `farmer` - Address performing emergency unstake
    /// * `pool_id` - Pool to unstake from
    /// * `amount` - Amount to unstake
    ///
    /// # Returns
    /// * `Result<i128, StakeError>` - Amount after penalty
    pub fn emergency_unstake(
        env: Env,
        farmer: Address,
        pool_id: BytesN<32>,
        amount: i128,
    ) -> Result<i128, StakeError> {
        staking::emergency_unstake(env, farmer, pool_id, amount)
    }

    /// Pause staking in a pool (admin only)
    ///
    /// # Arguments
    /// * `admin` - Address of the pool admin
    /// * `pool_id` - Pool to pause
    ///
    /// # Returns
    /// * `Result<(), PoolError>`
    pub fn pause_pool(env: Env, admin: Address, pool_id: BytesN<32>) -> Result<(), PoolError> {
        pool::pause_pool(env, admin, pool_id)
    }

    /// Unpause staking in a pool (admin only)
    ///
    /// # Arguments
    /// * `admin` - Address of the pool admin
    /// * `pool_id` - Pool to unpause
    ///
    /// # Returns
    /// * `Result<(), PoolError>`
    pub fn unpause_pool(env: Env, admin: Address, pool_id: BytesN<32>) -> Result<(), PoolError> {
        pool::unpause_pool(env, admin, pool_id)
    }
}

#[cfg(test)]
mod tests {
    pub mod pool;
    pub mod rewards;
    pub mod staking;
    pub mod utils;
}
