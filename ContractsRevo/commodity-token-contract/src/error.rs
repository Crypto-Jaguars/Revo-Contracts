use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    TokenNotFound = 3,
    OwnerNotFound = 4,
    InvalidInput = 5,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RedeemError {
    TokenNotFound = 1,
    NotTokenOwner = 2,
    InsufficientQuantity = 3,
    TokenExpired = 4,
    InventoryUnderflow = 5,
}

// Implement From for ContractError -> RedeemError
impl From<ContractError> for RedeemError {
    fn from(err: ContractError) -> Self {
        match err {
            ContractError::TokenNotFound => RedeemError::TokenNotFound,
            ContractError::OwnerNotFound => RedeemError::NotTokenOwner,
            _ => RedeemError::InventoryUnderflow,
        }
    }
}
