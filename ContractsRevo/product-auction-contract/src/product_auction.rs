use soroban_sdk::{contractimpl, Address, Env};

use crate::{
    datatype::{Auction, AuctionError, DataKeys, Product},
    interfaces::AuctionOperations,
    ProductAuctionContract, ProductAuctionContractArgs, ProductAuctionContractClient,
};

#[contractimpl]
impl AuctionOperations for ProductAuctionContract {
    fn create_auction(
        env: Env,
        seller: Address,
        reserve_price: u64,
        auction_end_time: u64,
        product_id: u64,
    ) -> Result<(), AuctionError> {
        seller.require_auth();
        let key = &DataKeys::Auction(seller.clone(), product_id);

        // Ensure auction does not already exist
        if env.storage().instance().has(key) {
            return Err(AuctionError::AuctionAlreadyExists);
        }

        // Create a new auction
        let auction = Auction {
            product_id,
            highest_bid: 0,
            highest_bidder: None,
            reserve_price,
            auction_end_time,
            seller: seller.clone(),
        };

        // Save the auction to storage
        env.storage().instance().set(key, &auction);

        env.events()
            .publish((seller.clone(), "AuctionCreated", seller.clone()), auction);

        Ok(())
    }

    fn place_bid(
        env: Env,
        product_id: u64,
        bid_amount: u64,
        bidder: Address,
        seller: Address,
    ) -> Result<bool, AuctionError> {
        let key = DataKeys::Auction(seller.clone(), product_id);

        // Fetch the auction from storage
        let mut auction: Auction = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(AuctionError::AuctionNotFound)?;

        // Ensure bidder is not the seller
        if bidder == auction.seller {
            return Err(AuctionError::InvalidBidder);
        }

        // Check if the auction has already ended
        let current_time = env.ledger().timestamp();
        if auction.auction_end_time < current_time {
            return Err(AuctionError::AuctionEnded);
        }

        // Ensure bid is higher than the current highest bid and meets the reserve price
        if bid_amount <= auction.highest_bid || bid_amount < auction.reserve_price {
            return Err(AuctionError::BidTooLow);
        }

        // Update auction state with the new highest bid
        auction.highest_bid = bid_amount;
        auction.highest_bidder = Some(bidder.clone());

        // Save the updated auction to storage
        env.storage().instance().set(&key, &auction);

        env.events().publish(
            (seller.clone(), "NewBid", product_id),
            &(bidder.clone(), bid_amount),
        );

        Ok(true)
    }

    fn extend_auction(
        env: Env,
        seller: Address,
        product_id: u64,
        new_end_time: u64,
    ) -> Result<(), AuctionError> {
        seller.require_auth();

        let key = DataKeys::Auction(seller.clone(), product_id);

        let mut auction: Auction = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(AuctionError::AuctionNotFound)?;

        let current_time = env.ledger().timestamp();

        // Ensure the auction hasn't ended already
        if auction.auction_end_time < current_time {
            return Err(AuctionError::AuctionEnded);
        }

        // Prevent last-minute extensions
        let min_extension_time = 600; // 10 minutes
        if auction.auction_end_time - current_time <= min_extension_time {
            return Err(AuctionError::TooLateToExtend);
        }

        // Ensure the new end time is in the future
        if new_end_time <= auction.auction_end_time {
            return Err(AuctionError::InvalidAuctionEndTime);
        }

        // Update auction end time
        auction.auction_end_time = new_end_time;

        // Save updated auction to storage
        env.storage().instance().set(&key, &auction);

        env.events().publish(
            (seller.clone(), "AuctionExtended", product_id),
            &new_end_time,
        );

        Ok(())
    }

    fn finalize_auction(env: Env, seller: Address, product_id: u64) -> Result<(), AuctionError> {
        seller.require_auth();

        let auction_key = DataKeys::Auction(seller.clone(), product_id.clone());

        // Fetch auction details
        let auction: Auction = env
            .storage()
            .instance()
            .get(&auction_key)
            .ok_or(AuctionError::AuctionNotFound)?;

        let current_time = env.ledger().timestamp();

        // Ensure auction has ended
        if auction.auction_end_time > current_time {
            return Err(AuctionError::AuctionNotYetEnded);
        }

        // Ensure there is a winning bidder
        let winner = auction.highest_bidder.ok_or(AuctionError::NoBidsPlaced)?;

        let product_key = DataKeys::Product(seller.clone(), product_id.clone());

        // Fetch product from storage
        let mut product: Product = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(AuctionError::ProductNotFound)?;

        // Ensure there is enough stock to fulfill the auction
        if product.stock == 0 {
            return Err(AuctionError::OutOfStock);
        }

        // Deduct product from inventory
        product.stock -= 1;

        // Update product storage
        env.storage().persistent().set(&product_key, &product);

        // Remove auction from storage (auction is complete)
        env.storage().instance().remove(&auction_key);

        // Emit event to notify that the auction is finalized
        env.events()
            .publish((seller.clone(), "AuctionFinalized", product.name), &winner);

        Ok(())
    }
}
