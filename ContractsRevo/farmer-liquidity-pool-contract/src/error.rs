use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PoolError {
    // General errors
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PoolNotActive = 4,
    
    // Token and liquidity errors
    InvalidToken = 5,
    InvalidTokenPair = 6,
    InvalidAmount = 7,
    InvalidLPTokenAmount = 8,
    InsufficientBalance = 9,
    InsufficientLiquidity = 10,
    InsufficientReserves = 11,
    
    // Fee and rate errors
    InvalidFeeRate = 12,
    SlippageExceeded = 13,
    
    // Math errors
    MathOverflow = 14,
    DivisionByZero = 15,
    
    // Swap errors
    SwapAmountTooLarge = 16,
    InvalidSwapDirection = 17,
    
    // Liquidity provision errors
    LiquidityRatioMismatch = 18,
    MinimumLiquidityNotMet = 19,
    
    // Fee distribution errors
    NoFeesToDistribute = 20,
    FeeDistributionFailed = 21,
}

