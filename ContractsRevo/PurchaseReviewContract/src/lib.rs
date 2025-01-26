#![no_std]
use soroban_sdk::contract;


mod rating;
mod review;
mod verification;
mod interface;
mod datatype;


#[cfg(test)]
mod test;

#[contract]
pub struct PurchaseReviewContract;
