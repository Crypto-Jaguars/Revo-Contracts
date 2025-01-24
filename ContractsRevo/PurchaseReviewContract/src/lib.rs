#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, Vec};
use crate::datatypes::{DataKey};


mod rating;
mod review;
mod verification;
mod interface;

pub use rating::Rating;
pub use review::Review;
pub use verification::Verification;


#[contract]
pub struct PurchaseReviewContract;
