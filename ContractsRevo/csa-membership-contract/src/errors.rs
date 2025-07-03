use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    /// Returned when a requested membership cannot be found
    NotFound = 1,
    /// Returned when provided dates are invalid (e.g., end date before start date)
    InvalidDates = 2,
    /// Returned when an operation is attempted by an unauthorized account
    NotAuthorized = 3,
    /// Returned when an invalid farm identifier is provided
    InvalidFarm = 4,
    /// Returned when an invalid season identifier is provided
    InvalidSeason = 5,
    /// Returned when an operation is attempted on an already cancelled membership
    AlreadyCancelled = 6,
}
