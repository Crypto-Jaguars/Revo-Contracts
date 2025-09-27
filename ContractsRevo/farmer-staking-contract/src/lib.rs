#![no_std]

use soroban_sdk::{
    contract, contractimpl, token, Address, Env,
};

pub mod storage;
pub mod types;
pub mod errors;

// #[cfg(test)]
// pub mod tests;

use storage::*;
use types::*;
use errors::*;

#[contract]
pub struct FarmerStakingContract;

#[contractimpl]
impl FarmerStakingContract {
    /// Initialize the staking pool
    pub fn initialize(
        env: Env,
        admin: Address,
        farmer_token: Address,
        reward_token: Address,
        min_stake_amount: u128,
        min_lock_period: u64,
    ) -> Result<(), StakingError> {
        admin.require_auth();
        
        if has_pool_info(&env) {
            return Err(StakingError::PoolAlreadyInitialized);
        }

        let pool_info = PoolInfo {
            admin,
            farmer_token,
            reward_token,
            min_stake_amount,
            min_lock_period,
            total_staked: 0,
            total_rewards: 0,
            last_reward_time: env.ledger().timestamp(),
        };

        set_pool_info(&env, &pool_info);
        Ok(())
    }

    /// Stake farmer tokens
    pub fn stake(
        env: Env,
        staker: Address,
        amount: u128,
        lock_period: u64,
    ) -> Result<(), StakingError> {
        staker.require_auth();

        let pool_info = get_pool_info(&env)?;
        
        if amount < pool_info.min_stake_amount {
            return Err(StakingError::InsufficientStakeAmount);
        }

        if lock_period < pool_info.min_lock_period {
            return Err(StakingError::InvalidLockPeriod);
        }

        // Transfer tokens from staker to contract
        let token_client = token::Client::new(&env, &pool_info.farmer_token);
        token_client.transfer(&staker, &env.current_contract_address(), &(amount as i128));

        let stake_info = StakeInfo {
            staker: staker.clone(),
            amount,
            lock_period,
            stake_time: env.ledger().timestamp(),
            last_reward_claim: env.ledger().timestamp(),
            rewards_earned: 0,
        };

        set_stake_info(&env, &staker, &stake_info);
        
        // Update pool totals
        let mut updated_pool = pool_info;
        updated_pool.total_staked += amount;
        set_pool_info(&env, &updated_pool);

        Ok(())
    }

    /// Calculate and distribute rewards
    pub fn calculate_rewards(env: Env, staker: Address) -> Result<u128, StakingError> {
        let stake_info = get_stake_info(&env, &staker)?;
        let pool_info = get_pool_info(&env)?;
        
        let current_time = env.ledger().timestamp();
        let time_staked = current_time - stake_info.last_reward_claim;
        
        // Simple reward calculation: 10% APY
        let annual_reward_rate = 10; // 10%
        let seconds_per_year = 365 * 24 * 60 * 60;
        
        let rewards = (stake_info.amount * annual_reward_rate * time_staked as u128) 
            / (100 * seconds_per_year as u128);
        
        Ok(rewards)
    }

    /// Claim rewards
    pub fn claim_rewards(env: Env, staker: Address) -> Result<u128, StakingError> {
        staker.require_auth();
        
        let rewards = Self::calculate_rewards(env.clone(), staker.clone())?;
        
        if rewards == 0 {
            return Err(StakingError::NoRewardsAvailable);
        }

        let pool_info = get_pool_info(&env)?;
        let mut stake_info = get_stake_info(&env, &staker)?;
        
        // Transfer reward tokens
        let reward_token_client = token::Client::new(&env, &pool_info.reward_token);
        reward_token_client.transfer(
            &env.current_contract_address(),
            &staker,
            &(rewards as i128)
        );

        // Update stake info
        stake_info.last_reward_claim = env.ledger().timestamp();
        stake_info.rewards_earned += rewards;
        set_stake_info(&env, &staker, &stake_info);

        Ok(rewards)
    }

    /// Unstake tokens
    pub fn unstake(env: Env, staker: Address) -> Result<(u128, u128), StakingError> {
        staker.require_auth();
        
        let stake_info = get_stake_info(&env, &staker)?;
        let pool_info = get_pool_info(&env)?;
        let current_time = env.ledger().timestamp();
        
        let lock_end_time = stake_info.stake_time + stake_info.lock_period;
        let is_early_unstake = current_time < lock_end_time;
        
        let mut amount_to_return = stake_info.amount;
        let mut slashing_penalty = 0u128;
        
        // Apply slashing for early unstaking (10% penalty)
        if is_early_unstake {
            slashing_penalty = amount_to_return / 10; // 10% penalty
            amount_to_return -= slashing_penalty;
        }

        // Calculate and claim any pending rewards
        let pending_rewards = Self::calculate_rewards(env.clone(), staker.clone())?;
        
        // Transfer staked tokens back
        let farmer_token_client = token::Client::new(&env, &pool_info.farmer_token);
        farmer_token_client.transfer(
            &env.current_contract_address(),
            &staker,
            &(amount_to_return as i128)
        );

        // Transfer rewards if any
        if pending_rewards > 0 {
            let reward_token_client = token::Client::new(&env, &pool_info.reward_token);
            reward_token_client.transfer(
                &env.current_contract_address(),
                &staker,
                &(pending_rewards as i128)
            );
        }

        // Update pool totals
        let mut updated_pool = pool_info;
        updated_pool.total_staked -= stake_info.amount;
        set_pool_info(&env, &updated_pool);

        // Remove stake info
        remove_stake_info(&env, &staker);

        Ok((amount_to_return, pending_rewards))
    }

    /// Get stake information
    pub fn get_stake_info(env: Env, staker: Address) -> Result<StakeInfo, StakingError> {
        get_stake_info(&env, &staker)
    }

    /// Get pool information
    pub fn get_pool_info(env: Env) -> Result<PoolInfo, StakingError> {
        get_pool_info(&env)
    }
}