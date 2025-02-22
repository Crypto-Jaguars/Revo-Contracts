use soroban_sdk::{contracterror, contracttype, Address, String, Symbol, Vec};

#[contracterror]
#[derive(Debug, Clone, PartialEq)]
    pub enum AuctionError {
        BidTooLow = 1,
        AuctionEnded = 2,
        AuctionAlreadyExists = 3,
        InvalidBidder = 4,
        AuctionNotFound = 5,
        TooLateToExtend = 6, 
        InvalidAuctionEndTime = 7,
        AuctionNotYetEnded = 8,
        NoBidsPlaced = 9,
        ProductNotFound = 10,
        OutOfStock = 11,
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
}

#[contracterror]
#[derive(Debug, Clone, PartialEq)]
pub enum ProductError {
    InvalidDescription=1,
    InvalidPrice=2,
    InvalidWeight=3,
    OutOfStock=4,
    InvalidImageCount=5,
    ProductNotFound=6,
    Unauthorized=7,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    New,
    OpenBox,
    UsedGood,
    UsedAcceptable,
    Refurbished,
}

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
