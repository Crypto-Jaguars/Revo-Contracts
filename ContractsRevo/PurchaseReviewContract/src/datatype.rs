use soroban_sdk::{
    Address,
    contracterror, contracttype, String
};


#[derive(Clone)]
#[contracttype]
pub enum DataKey {}

// Error definitions
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PurchaseReviewError {

}