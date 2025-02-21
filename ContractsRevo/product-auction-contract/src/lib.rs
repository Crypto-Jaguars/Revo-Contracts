#![no_std]
use datatype::{Auction, AuctionError, DataKeys};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

// use crate::datatype::{
//     DataKeys, ProductRatings, PurchaseReviewError, PurchaseVerificationData, ReviewDetails,
// };

mod product_auction;
mod listing;
mod shipping;
mod verification;
mod datatype;
mod interfaces;

#[cfg(test)]
mod test;

#[contract]
pub struct ProductAuctionContract;

#[contractimpl]
impl ProductAuctionContract {
    
    pub fn get_auction(env: Env, seller: Address, product_id: u128) -> Result<Auction, AuctionError> {
        let key = &DataKeys::Auction(seller.clone(), product_id);
        env.storage().instance().get(key).ok_or(AuctionError::AuctionNotFound)
    }
    
}
