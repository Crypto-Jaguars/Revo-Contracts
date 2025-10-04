use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    // Initialization Errors
    AlreadyInitialized = 1,

    // Authorization Errors
    Unauthorized = 2,
    NotInstructor = 3,

    // Data Errors
    ProgramNotFound = 4,
    ParticipantNotFound = 5,
    InvalidData = 6,

    // State Errors
    AlreadyEnrolled = 7,
    NotCompleted = 8,
    AlreadyCertified = 9,
}
