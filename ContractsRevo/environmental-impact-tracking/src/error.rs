use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Clone, PartialEq)]
pub enum ContractError {
    CreditNotFound = 1,
    ZeroAmount = 2,
    InvalidAmount = 3,
    InvalidIdentifier = 4,
    InvalidVerificationMethod = 5,
    CreditAlreadyExists = 6,
    AlreadyRetired = 7,
}
