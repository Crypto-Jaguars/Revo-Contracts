use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Error {
    ReputationHistoryNotFound = 1,
    RatingHistoryNotFound = 2,
}
