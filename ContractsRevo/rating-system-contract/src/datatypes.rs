use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Error {
    ReputaionHistoryNotFound = 1,
    RatingHistoryNotFound = 2,
}
