use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StakingError {
    PoolNotInitialized = 1,
    PoolAlreadyInitialized = 2,
    InsufficientStakeAmount = 3,
    InvalidLockPeriod = 4,
    StakeNotFound = 5,
    NoRewardsAvailable = 6,
    UnauthorizedAccess = 7,
    InvalidTokenAddress = 8,
    TransferFailed = 9,
}
