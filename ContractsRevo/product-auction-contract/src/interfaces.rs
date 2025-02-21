use soroban_sdk::{Address, Env};
use crate::datatype::AuctionError;

#[allow(dead_code)]
pub trait AuctionOperations {
    fn create_auction(env: Env, seller: Address, reserve_price: u64, auction_end_time: u64, product_id: u128) -> Result<(), AuctionError>;

    fn place_bid(
        env: Env,
        product_id: u128,
        bid_amount: u64,
        bidder: Address,
        seller: Address,
    ) -> Result<bool, AuctionError>;

    fn extend_auction(env: Env, seller: Address,product_id: u128, new_end_time: u64) -> Result<(), AuctionError>;
}
