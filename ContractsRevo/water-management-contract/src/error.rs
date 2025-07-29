use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Clone, PartialEq)]
pub enum ContractError {
    // General errors
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidInput = 4,

    // Water usage errors
    UsageNotFound = 10,
    UsageAlreadyExists = 11,
    InvalidVolume = 12,
    InvalidTimestamp = 13,
    InvalidDataHash = 14,

    // Threshold errors
    ThresholdNotFound = 20,
    InvalidThreshold = 21,
    ThresholdAlreadyExists = 22,

    // Incentive errors
    IncentiveNotFound = 30,
    IncentiveAlreadyExists = 31,
    InvalidRewardAmount = 32,
    InsufficientEfficiency = 33,

    // Alert errors
    AlertNotFound = 40,
    AlertAlreadyExists = 41,
    InvalidAlertType = 42,

    // Parcel and farmer errors
    InvalidParcelId = 50,
    InvalidFarmerId = 51,
    ParcelNotFound = 52,
    FarmerNotFound = 53,

    // Oracle and data errors
    OracleDataInvalid = 60,
    SensorDataCorrupted = 61,
    DataVerificationFailed = 62,
}
