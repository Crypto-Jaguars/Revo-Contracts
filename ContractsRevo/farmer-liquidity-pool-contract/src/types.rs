use soroban_sdk::{contracttype, Address};

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    TokenA = 0,
    TokenB = 1,
    TokenShare = 2,
    TotalShares = 3,
    ReserveA = 4,
    ReserveB = 5,
    Admin = 6,
    LPSupply = 7,
    FeeRate = 8,
    AccumulatedFeeA = 9,
    AccumulatedFeeB = 10,
    FeePerShareA = 11,
    FeePerShareB = 12,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Position {
    pub provider: Address,
    pub liquidity: i128,
    pub shares: i128,
    pub fee_debt_a: i128, // Tracks fees already accounted for token A
    pub fee_debt_b: i128, // Tracks fees already accounted for token B
}
