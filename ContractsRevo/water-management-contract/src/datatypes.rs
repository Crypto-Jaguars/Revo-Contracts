use soroban_sdk::{contracttype, Address, BytesN, String};

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct WaterUsage {
    pub usage_id: BytesN<32>,
    pub farmer_id: Address,
    pub parcel_id: BytesN<32>,
    pub volume: i128, // Water volume in liters
    pub timestamp: u64,
    pub data_hash: BytesN<32>, // Hash of off-chain sensor data
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Incentive {
    pub farmer_id: Address,
    pub reward_amount: i128,
    pub timestamp: u64,
    pub usage_id: BytesN<32>, // Reference to the water usage that earned the reward
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct WaterThreshold {
    pub parcel_id: BytesN<32>,
    pub daily_limit: i128, // Daily water limit in liters
    pub weekly_limit: i128, // Weekly water limit in liters
    pub monthly_limit: i128, // Monthly water limit in liters
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Alert {
    pub alert_id: BytesN<32>,
    pub farmer_id: Address,
    pub parcel_id: BytesN<32>,
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: u64,
    pub resolved: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum AlertType {
    ExcessiveUsage,
    ThresholdExceeded,
    SensorMalfunction,
    EfficiencyAlert,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct UsageReport {
    pub farmer_id: Address,
    pub parcel_id: Option<BytesN<32>>, // None for farmer-wide report
    pub total_usage: i128,
    pub period_start: u64,
    pub period_end: u64,
    pub efficiency_score: u32, // 0-100 efficiency rating
}

#[contracttype]
pub enum DataKey {
    Usage(BytesN<32>),
    Incentive(BytesN<32>),
    Threshold(BytesN<32>),
    Alert(BytesN<32>),
    FarmerUsages(Address),
    ParcelUsages(BytesN<32>),
    FarmerIncentives(Address),
    FarmerAlerts(Address), // Index of alert IDs for a farmer
    Admin,
}
