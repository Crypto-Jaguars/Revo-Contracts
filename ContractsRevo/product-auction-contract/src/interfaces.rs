use soroban_sdk::{Address, Env, String, Symbol, Vec};
use crate::datatype::{AuctionError, Condition, DisputeStatus, ProductError, ShippingError, VerificationError};

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

    fn finalize_auction(env: Env, seller: Address, product_id: u128) -> Result<(), AuctionError>;
}

#[allow(dead_code)]
pub trait ProductListing {
    fn add_product(
        env: Env,
        seller: Address,
        name: Symbol,
        description: String,
        price: u64,
        condition: Condition,
        stock: u32,
        images: Vec<String>,
        weight_grams: u64
    ) -> Result<u128, ProductError>; 

    fn update_stock(env: Env, seller: Address, product_id: u128, new_stock: u32) -> Result<(), ProductError>;
}

#[allow(dead_code)]
pub trait ShippingOperations {
    fn calculate_shipping_cost(weight_pounds: u32, distance_km: u32) -> u64;

    fn estimate_delivery_time(distance_km: u32) -> u32;

    fn create_shipment(
        env: Env,
        seller: Address,
        buyer: Address,
        buyer_zone: String,
        weight_grams: u32,
        distance_km: u32,
        tracking_number: String
    ) -> Result<String, ShippingError>;

    fn update_shipping_status(env: Env, tracking_number: String, seller: Address, new_status: Symbol) -> Result<(), ShippingError>;
}

#[allow(dead_code)]
pub trait VerificationOperations {
    fn verify_product(env: Env, admin: Address, seller: Address, product_id: u128, is_authentic: bool) -> Result<(), VerificationError>;

    fn request_seller_verification(env: Env, seller: Address) -> Result<(), VerificationError>;

    fn verify_seller(env: Env, admin: Address, seller: Address, is_verified: bool) -> Result<(), VerificationError>;

    fn verify_condition(env: Env, admin: Address, seller: Address, product_id: u128, condition: Condition) -> Result<(), VerificationError>;

    fn open_dispute(env: Env, buyer: Address, seller: Address, product_id: u128, reason: String) -> Result<(), VerificationError>;

    fn resolve_dispute(env: Env, admin: Address, seller: Address, product_id: u128, resolution: DisputeStatus) -> Result<(), VerificationError>;

    fn set_return_policy(env: Env, seller: Address, policy: String) -> Result<(), VerificationError>;

    fn request_return(env: Env, buyer: Address, seller: Address, product_id: u128, reason: String) -> Result<(), VerificationError>;
}