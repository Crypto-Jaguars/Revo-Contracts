use soroban_sdk::{contracterror, contracttype, Address, String, Symbol, Vec};

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
        InvalidAuctionEndTime = 8,
        AuctionNotYetEnded = 9,
        NoBidsPlaced = 10,
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

#[contracttype]
#[derive(Clone)]
pub enum DataKeys {
    Auction(Address, u128), // Sellers Created Auctions
    ProductList(Address), // ProductList of Seller
    Product(Address, u128), // Product related to Seller
    ProductCounter(Address), // Product Counter
}

#[contracterror]
#[derive(Debug, Clone, PartialEq)]
pub enum ProductError {
    InvalidDescription=1,
    InvalidCondition=2,
    InvalidPrice=3,
    InvalidWeight=4,
    OutOfStock=5,
    InvalidImageCount=6,
    ProductNotFound=7,
    Unauthorized=8,
}

// Condition categories for products
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    New,
    OpenBox,
    UsedGood,
    UsedAcceptable,
    Refurbished,
}

// Product structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct Product {
    pub id: u128,
    pub seller: Address,
    pub name: Symbol,
    pub description: String,
    pub price: u64,
    pub condition: Condition,
    pub stock: u32,
    pub images: Vec<String>,
    pub weight_grams: u64,
}