#![no_std]
use soroban_sdk::{contract, contractimpl,  token, Address, Env};
mod datatype;
use crate::datatype::*;


#[contract]
pub struct FarmerYieldFarmingContract;

#[contractimpl]
impl FarmerYieldFarmingContract {
    // ========== INITIALIZATION ==========
    pub fn initialize(env: Env, admin: Address)-> Result<bool,ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return  Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FarmCount, &0u32);
        env.storage().instance().set(&DataKey::GlobalMultiplier, &BASE_MULTIPLIER);
        env.storage().instance().set(&DataKey::MinStakePeriod, &COOLDOWN_PERIOD);
        env.storage().instance().set(&DataKey::EmergencyWithdraw, &false);
        env.storage().instance().extend_ttl(1000000, 1000000);
        Ok(true)
    }

    // ========== FARM MANAGEMENT ==========
    pub fn create_farm(
        env: Env,
        lp_token: Address,
        reward_token: Address,
        reward_per_block: i128,
        multiplier: u32,
        start_block: u64,
        end_block: u64,
    ) -> Result<u32, ContractError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if reward_per_block <= 0 || multiplier < BASE_MULTIPLIER || multiplier > MAX_MULTIPLIER {
            return Err(ContractError::InvalidParameters);
        }
        if start_block >= end_block || start_block < env.ledger().sequence() as u64 {
            return Err(ContractError::InvalidBlockRange);
        }

        let farm_id: u32 = env.storage().instance().get(&DataKey::FarmCount).unwrap_or(0);
        let farm = FarmPool {
            lp_token: lp_token.clone(),
            reward_token: reward_token.clone(),
            reward_per_block,
            total_staked: 0,
            multiplier,
            acc_reward_per_share: 0,
            last_reward_block: start_block,
            start_block,
            end_block,
            is_active: true,
        };

        env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
        env.storage().persistent().set(&DataKey::Paused(farm_id), &false);
        env.storage().instance().set(&DataKey::FarmCount, &(farm_id + 1));

        env.events().publish((soroban_sdk::symbol_short!("farm_new"),), (farm_id, lp_token, reward_token));
        Ok(farm_id)
    }

    pub fn update_farm(env: Env, farm_id: u32, reward_per_block: i128, multiplier: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut farm: FarmPool = env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap();
        Self::update_pool_internal(&env, farm_id);

        if reward_per_block > 0 {
            farm.reward_per_block = reward_per_block;
        }
        if multiplier >= BASE_MULTIPLIER && multiplier <= MAX_MULTIPLIER {
            farm.multiplier = multiplier;
        }

        env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
        env.events().publish((soroban_sdk::symbol_short!("farm_upd"),), (farm_id, reward_per_block, multiplier));
    }

    pub fn set_farm_paused(env: Env, farm_id: u32, paused: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Paused(farm_id), &paused);
        env.events().publish((soroban_sdk::symbol_short!("farm_paus"),), (farm_id, paused));
    }

    pub fn end_farm(env: Env, farm_id: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut farm: FarmPool = env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap();
        farm.end_block = env.ledger().sequence() as u64;
        farm.is_active = false;
        env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
        env.events().publish((soroban_sdk::symbol_short!("farm_end"),), farm_id);
    }

    // ========== STAKING OPERATIONS ==========
    pub fn stake_lp(env: Env, farmer: Address, farm_id: u32, amount: i128)-> Result<(), ContractError>{
        farmer.require_auth();

        if amount < MIN_STAKE_AMOUNT {
            return  Err(ContractError::AmountBelowMinimum);
        }

        let paused: bool = env.storage().persistent().get(&DataKey::Paused(farm_id)).unwrap_or(false);
        if paused {
           return  Err(ContractError::FarmPaused);
        }

        let mut farm: FarmPool = env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap();
        let current_block = env.ledger().sequence() as u64;

        if current_block < farm.start_block || current_block >= farm.end_block {
           return  Err(ContractError::FarmNotActive);
        }

        Self::update_pool_internal(&env, farm_id);

        let key = DataKey::UserFarm(farmer.clone(), farm_id);
        let mut user = env.storage().persistent().get(&key).unwrap_or(UserFarm {
            farmer: farmer.clone(),
            amount: 0,
            reward_debt: 0,
            stake_time: current_block,
            last_harvest: current_block,
        });

        if user.amount > 0 {
            let pending = Self::calc_pending(&env, &farm, &user);
            if pending > 0 {
               let _= Self::safe_transfer(&env, &farm.reward_token, &farmer, pending);
                env.events().publish((soroban_sdk::symbol_short!("harvest"),), (farmer.clone(), farm_id, pending));
            }
        }

        token::Client::new(&env, &farm.lp_token).transfer(&farmer, &env.current_contract_address(), &amount);

        user.amount += amount;
        user.reward_debt = (user.amount * farm.acc_reward_per_share) / PRECISION;
        user.stake_time = current_block;

        farm.total_staked += amount;

        env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
        env.storage().persistent().set(&key, &user);
        env.events().publish((soroban_sdk::symbol_short!("stake"),), (farmer, farm_id, amount));
        Ok(())
    }

    pub fn unstake_lp(env: Env, farmer: Address, farm_id: u32, amount: i128) -> Result<(), ContractError>{
        farmer.require_auth();

        if amount <= 0 {
            return  Err(ContractError::InvalidAmount);
        }

        let mut farm: FarmPool = env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap();
        let key = DataKey::UserFarm(farmer.clone(), farm_id);
        let mut user: UserFarm = env.storage().persistent().get(&key).unwrap();

        if amount > user.amount {
            return  Err(ContractError::InsufficientStake);
        }

        let current_block = env.ledger().sequence() as u64;
        let min_period: u64 = env.storage().instance().get(&DataKey::MinStakePeriod).unwrap_or(COOLDOWN_PERIOD);
        let time_staked = current_block.saturating_sub(user.stake_time);

        Self::update_pool_internal(&env, farm_id);

        let pending = Self::calc_pending(&env, &farm, &user);
        if pending > 0 {
            let actual_reward = if time_staked < min_period { pending / 2 } else { pending };
           let _= Self::safe_transfer(&env, &farm.reward_token, &farmer, actual_reward);
            env.events().publish((soroban_sdk::symbol_short!("harvest"),), (farmer.clone(), farm_id, actual_reward));
        }

        user.amount -= amount;
        user.reward_debt = (user.amount * farm.acc_reward_per_share) / PRECISION;
        farm.total_staked -= amount;

        token::Client::new(&env, &farm.lp_token).transfer(&env.current_contract_address(), &farmer, &amount);

        if user.amount == 0 {
            env.storage().persistent().remove(&key);
        } else {
            env.storage().persistent().set(&key, &user);
        }

        env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
        env.events().publish((soroban_sdk::symbol_short!("unstake"),), (farmer, farm_id, amount));
        Ok(())
    }

    pub fn harvest(env: Env, farmer: Address, farm_id: u32)-> Result<(), ContractError> {
        farmer.require_auth();

        let farm: FarmPool = env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap();
        let key = DataKey::UserFarm(farmer.clone(), farm_id);
        let mut user: UserFarm = env.storage().persistent().get(&key).unwrap();

        Self::update_pool_internal(&env, farm_id);

        let pending = Self::calc_pending(&env, &farm, &user);
        if pending <= 0 {
           return  Err(ContractError::NoRewards);
        }

        let _= Self::safe_transfer(&env, &farm.reward_token, &farmer, pending);

        user.reward_debt = (user.amount * farm.acc_reward_per_share) / PRECISION;
        user.last_harvest = env.ledger().sequence() as u64;

        env.storage().persistent().set(&key, &user);
        env.events().publish((soroban_sdk::symbol_short!("harvest"),), (farmer, farm_id, pending));
        Ok(())
    }

    pub fn emergency_withdraw(env: Env, farmer: Address, farm_id: u32)-> Result<(), ContractError> {
        farmer.require_auth();

        let enabled: bool = env.storage().instance().get(&DataKey::EmergencyWithdraw).unwrap_or(false);
        if !enabled {
           return  Err(ContractError::EmergencyNotEnabled);
        }

        let mut farm: FarmPool = env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap();
        let key = DataKey::UserFarm(farmer.clone(), farm_id);
        let user: UserFarm = env.storage().persistent().get(&key).unwrap();

        let amount = user.amount;
        token::Client::new(&env, &farm.lp_token).transfer(&env.current_contract_address(), &farmer, &amount);

        farm.total_staked -= amount;
        env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
        env.storage().persistent().remove(&key);

        env.events().publish((soroban_sdk::symbol_short!("emerg_wd"),), (farmer, farm_id, amount));
        Ok(())
    }

    // ========== REWARD QUERIES ==========
    pub fn get_pending_rewards(env: Env, farmer: Address, farm_id: u32) -> i128 {
        let farm: FarmPool = match env.storage().persistent().get(&DataKey::Farm(farm_id)) {
            Some(f) => f,
            None => return 0,
        };

        let user: UserFarm = match env.storage().persistent().get(&DataKey::UserFarm(farmer, farm_id)) {
            Some(u) => u,
            None => return 0,
        };

        Self::calc_pending(&env, &farm, &user)
    }

    fn calc_pending(env: &Env, farm: &FarmPool, user: &UserFarm) -> i128 {
        if user.amount == 0 {
            return 0;
        }

        let mut acc = farm.acc_reward_per_share;
        let current_block = env.ledger().sequence() as u64;

        if current_block > farm.last_reward_block && farm.total_staked > 0 {
            let end_block = if current_block > farm.end_block { farm.end_block } else { current_block };
            let blocks = (end_block - farm.last_reward_block) as i128;
            let global_mult: u32 = env.storage().instance().get(&DataKey::GlobalMultiplier).unwrap_or(BASE_MULTIPLIER);
            let total_mult = (farm.multiplier as i128 * global_mult as i128) / BASE_MULTIPLIER as i128;
            let reward = (blocks * farm.reward_per_block * total_mult) / BASE_MULTIPLIER as i128;
            acc += (reward * PRECISION) / farm.total_staked;
        }

        let tier = Self::get_tier(user.amount);
        let tier_mult = match tier {
            FarmerTier::Smallholder => 120,
            FarmerTier::Cooperative => 110,
            FarmerTier::Enterprise => 100,
        };

        let base = (user.amount * acc) / PRECISION - user.reward_debt;
        let with_tier = (base * tier_mult as i128) / 100;

        let time_staked = current_block.saturating_sub(user.stake_time);
        let loyalty = Self::get_loyalty_bonus(time_staked);

        with_tier + (with_tier * loyalty as i128) / 10000
    }

    fn get_tier(amount: i128) -> FarmerTier {
        if amount < 1_000_0000000 {
            FarmerTier::Smallholder
        } else if amount < 10_000_0000000 {
            FarmerTier::Cooperative
        } else {
            FarmerTier::Enterprise
        }
    }

    fn get_loyalty_bonus(time: u64) -> u32 {
        const DAY: u64 = 17280;
        if time < DAY * 7 { 0 }
        else if time < DAY * 30 { 500 }
        else if time < DAY * 90 { 1000 }
        else if time < DAY * 180 { 1500 }
        else { 2000 }
    }

    // ========== POOL UPDATES ==========
    pub fn update_pool(env: Env, farm_id: u32) {
        Self::update_pool_internal(&env, farm_id);
    }

    fn update_pool_internal(env: &Env, farm_id: u32) {
        let mut farm: FarmPool = env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap();
        let current = env.ledger().sequence() as u64;

        if current <= farm.last_reward_block || farm.total_staked == 0 {
            farm.last_reward_block = current;
            env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
            return;
        }

        let end_block = if current > farm.end_block { farm.end_block } else { current };
        let blocks = (end_block - farm.last_reward_block) as i128;
        let global_mult: u32 = env.storage().instance().get(&DataKey::GlobalMultiplier).unwrap_or(BASE_MULTIPLIER);
        let total_mult = (farm.multiplier as i128 * global_mult as i128) / BASE_MULTIPLIER as i128;
        let reward = (blocks * farm.reward_per_block * total_mult) / BASE_MULTIPLIER as i128;

        farm.acc_reward_per_share += (reward * PRECISION) / farm.total_staked;
        farm.last_reward_block = end_block;

        env.storage().persistent().set(&DataKey::Farm(farm_id), &farm);
    }

    // ========== UTILITY FUNCTIONS ==========
    fn safe_transfer(env: &Env, token: &Address, to: &Address, amount: i128)-> Result<bool, ContractError> {
        if amount <= 0 { return  Err(ContractError::InvalidAmount); }
        let client = token::Client::new(env, token);
        let balance = client.balance(&env.current_contract_address());
        if balance < amount { return Err(ContractError::InsufficientBalance) }
        client.transfer(&env.current_contract_address(), to, &amount);
        Ok(true)
    }

    pub fn get_farm(env: Env, farm_id: u32) -> FarmPool {
        env.storage().persistent().get(&DataKey::Farm(farm_id)).unwrap()
    }

    pub fn get_user_farm(env: Env, farmer: Address, farm_id: u32) -> Option<UserFarm> {
        env.storage().persistent().get(&DataKey::UserFarm(farmer, farm_id))
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn get_farm_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::FarmCount).unwrap_or(0)
    }

    pub fn set_global_multiplier(env: Env, multiplier: u32)-> Result<(), ContractError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if multiplier < BASE_MULTIPLIER || multiplier > MAX_MULTIPLIER { return  Err(ContractError::InvalidMultiplier); }
        env.storage().instance().set(&DataKey::GlobalMultiplier, &multiplier);
        Ok(())
    }

    pub fn set_emergency_withdraw(env: Env, enabled: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::EmergencyWithdraw, &enabled);
    }

    pub fn deposit_rewards(env: Env, token: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        token::Client::new(&env, &token).transfer(&admin, &env.current_contract_address(), &amount);
    }
}

#[cfg(test)]
mod test;
mod utils;

// Farm pool creation and management tests
mod farming;

// LP token staking and validation tests
mod staking;

// Reward harvesting and distribution tests
mod rewards;