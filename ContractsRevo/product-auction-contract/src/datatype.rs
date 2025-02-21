use soroban_sdk::{contracterror, contracttype, Address};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
    pub enum AuctionError {
        InvalidProductId = 1,
        BidTooLow = 2 ,
        AuctionEnded = 3,
        AuctionAlreadyExists = 4,
        InvalidBidder = 5,
        AuctionNotFound = 6,
        TooLateToExtend = 7, 
        InvalidAuctionEndTime = 8
    }

#[contracttype]
#[derive(Clone)]
pub struct Auction {
    pub product_id: u128,
    pub highest_bid: u64,
    pub highest_bidder: Option<Address>,
    pub reserve_price: u64,
    pub auction_end_time: u64,
    pub seller: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKeys {
    Auction(Address, u128) // Sellers Created Auctions
}