use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    // Initialization and Configuration Errors
    AlreadyInitialized = 1,
    Unauthorized = 2,
    OracleNotSet = 3,

    // Data Errors
    ProductNotFound = 4,
    ForecastNotFound = 5,
    InvalidData = 6,
    RegionNotFound = 7,

    // Hashing Errors
    HashError = 8,
}
