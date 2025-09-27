use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PoolError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidTokenPair = 4,
    InsufficientLiquidity = 5,
    InsufficientBalance = 6,
    InvalidAmount = 7,
    SlippageExceeded = 8,
    InvalidFeeRate = 9,
    ZeroAmount = 10,
    PoolNotActive = 11,
    InvalidToken = 12,
    MathOverflow = 13,
    InsufficientReserves = 14,
    InvalidLPTokenAmount = 15,
    LiquidityTooLow = 16,
}
