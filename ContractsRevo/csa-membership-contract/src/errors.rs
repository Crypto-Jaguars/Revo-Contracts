use soroban_sdk::{contracterror};

#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractError {
    MembershipNotFound = 1,
    NotAuthorized = 2,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Error {
    NotFound = 1,
    InvalidDates = 2,
    NotAuthorized = 3,
    InvalidFarm = 4,
    InvalidSeason = 5,
    AlreadyCancelled = 6,
}