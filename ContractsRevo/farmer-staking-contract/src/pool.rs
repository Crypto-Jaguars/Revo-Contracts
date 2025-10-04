use soroban_sdk::{contracterror, contracttype, Address, Bytes, BytesN, Env, Symbol, Vec};

/// Errors that can occur in pool operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PoolError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidRewardRate = 4,
    InvalidMinStake = 5,
    InvalidLockPeriod = 6,
    PoolNotFound = 7,
    PoolPaused = 8,
    PoolNotPaused = 9,
}

/// Staking pool configuration and state
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardPool {
    pub pool_id: BytesN<32>,
    pub admin: Address,
    pub token_address: Address,
    pub total_staked: i128,
    pub reward_rate: i128,
    pub current_epoch: u64,
    pub min_stake_amount: i128,
    pub max_lock_period: u64,
    pub is_paused: bool,
    pub created_at: u64,
    pub last_reward_update: u64,
}

/// Storage keys for pool data
#[contracttype]
#[derive(Clone)]
pub enum PoolStorageKey {
    Pool(BytesN<32>),
    PoolList,
    PoolCount,
}

/// Initialize a new staking pool
pub fn initialize_pool(
    env: Env,
    admin: Address,
    token_address: Address,
    reward_rate: i128,
    min_stake_amount: i128,
    max_lock_period: u64,
) -> Result<BytesN<32>, PoolError> {
    admin.require_auth();

    // Validate inputs
    if reward_rate <= 0 {
        return Err(PoolError::InvalidRewardRate);
    }
    if min_stake_amount <= 0 {
        return Err(PoolError::InvalidMinStake);
    }
    if max_lock_period == 0 {
        return Err(PoolError::InvalidLockPeriod);
    }

    // Generate unique pool ID
    let pool_count: u64 = env
        .storage()
        .instance()
        .get(&PoolStorageKey::PoolCount)
        .unwrap_or(0);

    // Create a unique identifier by combining pool count and timestamp
    let mut data = Bytes::new(&env);
    data.extend_from_slice(&pool_count.to_be_bytes());
    data.extend_from_slice(&env.ledger().timestamp().to_be_bytes());

    let pool_id: BytesN<32> = env.crypto().sha256(&data).into();

    // Check if pool already exists
    if env
        .storage()
        .instance()
        .has(&PoolStorageKey::Pool(pool_id.clone()))
    {
        return Err(PoolError::AlreadyInitialized);
    }

    let current_time = env.ledger().timestamp();

    let pool = RewardPool {
        pool_id: pool_id.clone(),
        admin: admin.clone(),
        token_address,
        total_staked: 0,
        reward_rate,
        current_epoch: 0,
        min_stake_amount,
        max_lock_period,
        is_paused: false,
        created_at: current_time,
        last_reward_update: current_time,
    };

    // Store pool data
    env.storage()
        .instance()
        .set(&PoolStorageKey::Pool(pool_id.clone()), &pool);

    // Update pool list
    let mut pool_list: Vec<BytesN<32>> = env
        .storage()
        .instance()
        .get(&PoolStorageKey::PoolList)
        .unwrap_or(Vec::new(&env));
    pool_list.push_back(pool_id.clone());
    env.storage()
        .instance()
        .set(&PoolStorageKey::PoolList, &pool_list);

    // Update pool count
    env.storage()
        .instance()
        .set(&PoolStorageKey::PoolCount, &(pool_count + 1));

    // Log event
    env.events()
        .publish((Symbol::new(&env, "pool_created"), admin), pool_id.clone());

    Ok(pool_id)
}

/// Get pool information
pub fn get_pool_info(env: Env, pool_id: BytesN<32>) -> Result<RewardPool, PoolError> {
    env.storage()
        .instance()
        .get(&PoolStorageKey::Pool(pool_id))
        .ok_or(PoolError::PoolNotFound)
}

/// Get all pool IDs
pub fn get_all_pools(env: Env) -> Vec<BytesN<32>> {
    env.storage()
        .instance()
        .get(&PoolStorageKey::PoolList)
        .unwrap_or(Vec::new(&env))
}

/// Update reward rate for a pool
pub fn update_reward_rate(
    env: Env,
    admin: Address,
    pool_id: BytesN<32>,
    new_reward_rate: i128,
) -> Result<(), PoolError> {
    admin.require_auth();

    if new_reward_rate <= 0 {
        return Err(PoolError::InvalidRewardRate);
    }

    let mut pool: RewardPool = get_pool_info(env.clone(), pool_id.clone())?;

    if pool.admin != admin {
        return Err(PoolError::Unauthorized);
    }

    pool.reward_rate = new_reward_rate;
    pool.last_reward_update = env.ledger().timestamp();

    env.storage()
        .instance()
        .set(&PoolStorageKey::Pool(pool_id.clone()), &pool);

    env.events().publish(
        (Symbol::new(&env, "reward_rate_updated"), admin),
        (pool_id, new_reward_rate),
    );

    Ok(())
}

/// Get total staked amount in a pool
pub fn get_total_staked(env: Env, pool_id: BytesN<32>) -> Result<i128, PoolError> {
    let pool = get_pool_info(env, pool_id)?;
    Ok(pool.total_staked)
}

/// Update total staked amount (internal function)
pub fn update_total_staked(
    env: Env,
    pool_id: BytesN<32>,
    amount_delta: i128,
) -> Result<(), PoolError> {
    let mut pool = get_pool_info(env.clone(), pool_id.clone())?;
    pool.total_staked = pool.total_staked.checked_add(amount_delta).unwrap_or(0);

    env.storage()
        .instance()
        .set(&PoolStorageKey::Pool(pool_id), &pool);

    Ok(())
}

/// Update epoch (internal function)
pub fn update_epoch(env: Env, pool_id: BytesN<32>) -> Result<(), PoolError> {
    let mut pool = get_pool_info(env.clone(), pool_id.clone())?;

    let current_time = env.ledger().timestamp();
    let time_elapsed = current_time - pool.last_reward_update;

    // Each epoch is 1 day (86400 seconds)
    let epochs_passed = time_elapsed / 86400;

    if epochs_passed > 0 {
        pool.current_epoch = pool
            .current_epoch
            .checked_add(epochs_passed)
            .unwrap_or(pool.current_epoch);
        pool.last_reward_update = current_time;

        env.storage()
            .instance()
            .set(&PoolStorageKey::Pool(pool_id), &pool);
    }

    Ok(())
}

/// Pause a pool (admin only)
pub fn pause_pool(env: Env, admin: Address, pool_id: BytesN<32>) -> Result<(), PoolError> {
    admin.require_auth();

    let mut pool = get_pool_info(env.clone(), pool_id.clone())?;

    if pool.admin != admin {
        return Err(PoolError::Unauthorized);
    }

    if pool.is_paused {
        return Err(PoolError::PoolPaused);
    }

    pool.is_paused = true;

    env.storage()
        .instance()
        .set(&PoolStorageKey::Pool(pool_id.clone()), &pool);

    env.events()
        .publish((Symbol::new(&env, "pool_paused"), admin), pool_id);

    Ok(())
}

/// Unpause a pool (admin only)
pub fn unpause_pool(env: Env, admin: Address, pool_id: BytesN<32>) -> Result<(), PoolError> {
    admin.require_auth();

    let mut pool = get_pool_info(env.clone(), pool_id.clone())?;

    if pool.admin != admin {
        return Err(PoolError::Unauthorized);
    }

    if !pool.is_paused {
        return Err(PoolError::PoolNotPaused);
    }

    pool.is_paused = false;

    env.storage()
        .instance()
        .set(&PoolStorageKey::Pool(pool_id.clone()), &pool);

    env.events()
        .publish((Symbol::new(&env, "pool_unpaused"), admin), pool_id);

    Ok(())
}

/// Check if pool is paused
pub fn is_pool_paused(env: Env, pool_id: BytesN<32>) -> Result<bool, PoolError> {
    let pool = get_pool_info(env, pool_id)?;
    Ok(pool.is_paused)
}
