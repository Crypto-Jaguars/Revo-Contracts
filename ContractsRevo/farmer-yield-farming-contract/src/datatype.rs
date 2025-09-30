use soroban_sdk::{ contracterror, contracttype, Address };

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    FarmCount,
    Farm(u32),
    UserFarm(Address, u32),
    Paused(u32),
    GlobalMultiplier,
    MinStakePeriod,
    EmergencyWithdraw,
}

#[derive(Clone)]
#[contracttype]
pub struct FarmPool {
    pub lp_token: Address,
    pub reward_token: Address,
    pub reward_per_block: i128,
    pub total_staked: i128,
    pub multiplier: u32,
    pub acc_reward_per_share: i128,
    pub last_reward_block: u64,
    pub start_block: u64,
    pub end_block: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct UserFarm {
    pub farmer: Address,
    pub amount: i128,
    pub reward_debt: i128,
    pub stake_time: u64,
    pub last_harvest: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum FarmerTier {
    Smallholder,
    Cooperative,
    Enterprise,
}

#[contracterror]
pub enum ContractError {
    AlreadyInitialized = 1,
    InvalidParameters = 2,
    InvalidBlockRange = 3,
    FarmPaused = 4,
    FarmNotActive = 5,
    AmountBelowMinimum = 6,
    InsufficientStake = 7,
    InvalidAmount = 8,
    NoRewards = 9,
    EmergencyNotEnabled = 10,
    FarmNotFound = 11,
    NoStakeFound = 12,
    InsufficientBalance = 13,
    InvalidMultiplier = 14,
    NotInitialized = 15,
}


pub const PRECISION: i128 = 1_000_000_000_000;
pub const MIN_STAKE_AMOUNT: i128 = 100;
pub const COOLDOWN_PERIOD: u64 = 86400;
pub const MAX_MULTIPLIER: u32 = 500;
pub const BASE_MULTIPLIER: u32 = 100;