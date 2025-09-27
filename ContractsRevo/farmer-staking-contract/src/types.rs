use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolInfo {
    pub admin: Address,
    pub farmer_token: Address,
    pub reward_token: Address,
    pub min_stake_amount: u128,
    pub min_lock_period: u64,
    pub total_staked: u128,
    pub total_rewards: u128,
    pub last_reward_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeInfo {
    pub staker: Address,
    pub amount: u128,
    pub lock_period: u64,
    pub stake_time: u64,
    pub last_reward_claim: u64,
    pub rewards_earned: u128,
}