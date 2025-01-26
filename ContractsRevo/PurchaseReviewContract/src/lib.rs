#![no_std]
use soroban_sdk::contract;


mod rating;
mod test;
mod review;
mod verification;
mod interface;
mod datatype;



#[contract]
pub struct PurchaseReviewContract;
