use soroban_sdk::{contractimpl, Address, Env};

use crate::{datatype::{Auction, AuctionError, DataKeys}, interfaces::AuctionOperations, ProductAuctionContract, ProductAuctionContractArgs, ProductAuctionContractClient};

#[contractimpl]
impl AuctionOperations for ProductAuctionContract {
    fn create_auction(env: Env, seller: Address, reserve_price: u64, auction_end_time: u64, product_id: u128) -> Result<(), AuctionError> {
        let key = &DataKeys::Auction(seller.clone(), product_id);

        // Ensure an auction does not exist for the given product ID
        let existing_auction: Option<Auction> = env.storage().instance().get(key);

        if let Some(existing_auction) = existing_auction {
            if existing_auction.product_id == product_id {
            return Err(AuctionError::AuctionAlreadyExists);
            }
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

        env.events().publish((seller.clone(), "AuctionCreated", seller.clone()), auction);
        
        Ok(())
    }
    
    fn place_bid(
        env: Env,
        product_id: u128,
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
    
        env.events().publish((seller.clone(), "NewBid", product_id), &(bidder.clone(), bid_amount));
    
        Ok(true)
    }

    fn extend_auction(env: Env, seller: Address, product_id: u128, new_end_time: u64) -> Result<(), AuctionError> {
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
        if auction.auction_end_time - current_time < min_extension_time {
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
    
        env.events().publish((seller.clone(), "AuctionExtended", product_id), &new_end_time);
    
        Ok(())
    }
    
}