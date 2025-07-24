use soroban_sdk::{contracttype, contracterror, Address, String, BytesN};

#[derive(Debug)]
#[contracterror]
pub enum StabilizationError {
    FundNotFound = 1,
    FundAlreadyExists = 2,
    InsufficientFunds = 3,
    Unauthorized = 4,
    InvalidInput = 5,
    OracleNotRegistered = 6,
    PriceDataNotAvailable = 7,
    FarmerNotRegistered = 8,
    FarmerAlreadyRegistered = 9,
    NoPayoutNeeded = 10,
    PayoutAlreadyProcessed = 11,
    ThresholdNotReached = 12,
    ChainlinkFeedNotFound = 13,
    StalePriceData = 14,
    InvalidChainlinkResponse = 15,
    ChainlinkFeedNotRegistered = 16,
    ChainlinkFeedAlreadyRegistered = 17,
    CropAlreadyRegistered = 18,
}

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    Admin,
    Fund(BytesN<32>),
    FundCounter,
    Contributor(BytesN<32>, Address),
    PriceOracle(String),
    MarketPrice(String),
    Farmer(Address),
    FarmerCrops(Address, String),
    Payout(BytesN<32>, Address, u64),
    PayoutCounter(BytesN<32>, Address),
    ChainlinkFeed(String),
    ChainlinkPrice(String),
}

#[contracttype]
pub struct StabilizationFund {
    pub fund_id: BytesN<32>,
    pub fund_name: String,
    pub admin: Address,
    pub total_balance: i128,
    pub price_threshold: i128,
    pub crop_type: String,
    pub active: bool,
    pub creation_time: u64,
    pub last_payout_time: Option<u64>,
}

#[contracttype]
pub struct Contributor {
    pub address: Address,
    pub total_contribution: i128,
    pub last_contribution_time: u64,
}

#[contracttype]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
    pub oracle: Address,
}

#[contracttype]
pub struct Farmer {
    pub address: Address,
    pub registered_time: u64,
    pub total_received_payouts: i128,
    pub active: bool,
}

#[contracttype]
pub struct FarmerCrop {
    pub crop_type: String,
    pub production_capacity: i128,
}

#[contracttype]
pub struct Payout {
    pub farmer_id: Address,
    pub fund_id: BytesN<32>,
    pub amount: i128,
    pub timestamp: u64,
    pub market_price: i128,
    pub threshold_price: i128,
}

#[contracttype]
pub struct ChainlinkPriceFeed {
    pub feed_address: Address,
    pub decimals: u32,
    pub description: String,
    pub crop_type: String,
    pub registered_time: u64,
    pub active: bool,
}

#[contracttype]
pub struct ChainlinkPriceData {
    pub price: i128,
    pub timestamp: u64,
    pub feed_address: Address,
    pub round_id: u64,
    pub decimals: u32,
}