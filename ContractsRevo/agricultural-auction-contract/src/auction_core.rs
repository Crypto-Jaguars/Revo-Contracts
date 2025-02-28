use soroban_sdk::{contractimpl, Address, Env, Symbol};

use crate::{
    AgriculturalAuctionContract, AgriculturalAuctionContractArgs,
    AgriculturalAuctionContractClient, AgriculturalProduct, Auction, AuctionError, DataKey,
};

pub trait AuctionOperations {
    fn create_auction(
        env: Env,
        farmer: Address,
        product_id: u64,
        reserve_price: u64,
        auction_end_time: u64,
        min_quantity: u32,
        bulk_discount_threshold: u32,
        bulk_discount_percentage: u32,
        dynamic_pricing: bool,
    ) -> Result<(), AuctionError>;

    fn place_bid(
        env: Env,
        product_id: u64,
        bid_amount: u64,
        bid_quantity: u32,
        bidder: Address,
        farmer: Address,
    ) -> Result<bool, AuctionError>;

    fn extend_auction(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_end_time: u64,
    ) -> Result<(), AuctionError>;

    fn finalize_auction(env: Env, farmer: Address, product_id: u64) -> Result<(), AuctionError>;
}

#[contractimpl]
impl AuctionOperations for AgriculturalAuctionContract {
    fn create_auction(
        env: Env,
        farmer: Address,
        product_id: u64,
        reserve_price: u64,
        auction_end_time: u64,
        min_quantity: u32,
        bulk_discount_threshold: u32,
        bulk_discount_percentage: u32,
        dynamic_pricing: bool,
    ) -> Result<(), AuctionError> {
        farmer.require_auth();

        let key = &DataKey::Auction(farmer.clone(), product_id);

        // Ensure auction does not already exist
        if env.storage().instance().has(key) {
            return Err(AuctionError::AuctionAlreadyExists);
        }

        // Get product details
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(AuctionError::ProductNotFound)?;

        // Check if product is expired
        let current_time = env.ledger().timestamp();
        if product.expiry_date <= current_time {
            return Err(AuctionError::ProductExpired);
        }

        // Ensure auction end time is before expiry date
        if auction_end_time >= product.expiry_date {
            return Err(AuctionError::InvalidAuctionEndTime);
        }

        // Validate auction end time is in the future
        if auction_end_time <= current_time {
            return Err(AuctionError::InvalidAuctionEndTime);
        }

        // Validate min quantity
        if min_quantity == 0 || min_quantity > product.quantity {
            return Err(AuctionError::QuantityUnavailable);
        }

        // Create a new auction
        let auction = Auction {
            product_id,
            highest_bid: 0,
            highest_bidder: None,
            reserve_price,
            auction_end_time,
            farmer: farmer.clone(),
            quantity_available: product.quantity,
            min_quantity,
            bulk_discount_threshold,
            bulk_discount_percentage,
            dynamic_pricing,
        };

        // Save the auction to storage
        env.storage().instance().set(key, &auction);

        // Emit event for auction creation
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "AuctionCreated"),
                product_id,
            ),
            auction,
        );

        Ok(())
    }

    fn place_bid(
        env: Env,
        product_id: u64,
        bid_amount: u64,
        bid_quantity: u32,
        bidder: Address,
        farmer: Address,
    ) -> Result<bool, AuctionError> {
        bidder.require_auth();

        let key = DataKey::Auction(farmer.clone(), product_id);

        // Fetch the auction from storage
        let mut auction: Auction = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(AuctionError::AuctionNotFound)?;

        // Get product details
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(AuctionError::ProductNotFound)?;

        // Ensure bidder is not the farmer
        if bidder == auction.farmer {
            return Err(AuctionError::InvalidBidder);
        }

        // Check if the auction has already ended
        let current_time = env.ledger().timestamp();
        if auction.auction_end_time < current_time {
            return Err(AuctionError::AuctionEnded);
        }

        // Check if product has expired during auction
        if product.expiry_date <= current_time {
            return Err(AuctionError::ProductExpired);
        }

        // Check if requested quantity is available and meets minimum
        if bid_quantity > auction.quantity_available || bid_quantity < auction.min_quantity {
            return Err(AuctionError::QuantityUnavailable);
        }

        // Calculate per-unit bid
        let per_unit_bid = bid_amount / (bid_quantity as u64);

        // Apply bulk discount if applicable
        let effective_bid = if bid_quantity >= auction.bulk_discount_threshold
            && auction.bulk_discount_percentage > 0
        {
            // Calculate effective bid with discount
            bid_amount - (bid_amount * auction.bulk_discount_percentage as u64 / 100)
        } else {
            bid_amount
        };

        // Ensure bid is higher than the current highest bid and meets the reserve price
        if effective_bid <= auction.highest_bid || per_unit_bid < auction.reserve_price {
            return Err(AuctionError::BidTooLow);
        }

        // Update auction state with the new highest bid
        auction.highest_bid = effective_bid;
        auction.highest_bidder = Some(bidder.clone());

        // Save the updated auction to storage
        env.storage().instance().set(&key, &auction);

        // Emit event for new bid
        env.events().publish(
            (farmer.clone(), Symbol::new(&env, "NewBid"), product_id),
            &(bidder.clone(), effective_bid, bid_quantity),
        );

        Ok(true)
    }

    fn extend_auction(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_end_time: u64,
    ) -> Result<(), AuctionError> {
        farmer.require_auth();

        let key = DataKey::Auction(farmer.clone(), product_id);

        let mut auction: Auction = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(AuctionError::AuctionNotFound)?;

        // Check if the auction has already ended
        let current_time = env.ledger().timestamp();
        if auction.auction_end_time < current_time {
            return Err(AuctionError::AuctionEnded);
        }

        // Get product details to verify expiry
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(AuctionError::ProductNotFound)?;

        // Check if product has expired
        if product.expiry_date <= current_time {
            return Err(AuctionError::ProductExpired);
        }

        // Can only extend before the auction ends
        if auction.auction_end_time <= current_time {
            return Err(AuctionError::TooLateToExtend);
        }

        // Ensure new end time is later than current end time
        if new_end_time <= auction.auction_end_time {
            return Err(AuctionError::InvalidAuctionEndTime);
        }

        // Ensure new end time is before product expiry
        if new_end_time >= product.expiry_date {
            return Err(AuctionError::InvalidAuctionEndTime);
        }

        // Update the auction end time
        auction.auction_end_time = new_end_time;

        // Save the updated auction
        env.storage().instance().set(&key, &auction);

        // Emit event for auction extension
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "AuctionExtended"),
                product_id,
            ),
            new_end_time,
        );

        Ok(())
    }

    fn finalize_auction(env: Env, farmer: Address, product_id: u64) -> Result<(), AuctionError> {
        farmer.require_auth();

        let key = DataKey::Auction(farmer.clone(), product_id);

        // Get the auction details
        let auction: Auction = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(AuctionError::AuctionNotFound)?;

        // Ensure the auction has ended
        let current_time = env.ledger().timestamp();
        if auction.auction_end_time > current_time {
            return Err(AuctionError::AuctionNotYetEnded);
        }

        // Check if there were any bids
        if auction.highest_bidder.is_none() {
            return Err(AuctionError::NoBidsPlaced);
        }

        // Get the product to update quantity
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let mut product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(AuctionError::ProductNotFound)?;

        // Update product quantity (assuming the entire available quantity was sold)
        product.quantity = product.quantity.saturating_sub(auction.quantity_available);

        // Save the updated product
        env.storage().persistent().set(&product_key, &product);

        // Remove the auction (or mark as completed)
        env.storage().instance().remove(&key);

        // Emit event for auction finalization
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "AuctionFinalized"),
                product_id,
            ),
            (auction.highest_bidder.unwrap(), auction.highest_bid),
        );

        Ok(())
    }
}