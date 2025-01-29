use crate::interface::{ReviewOperations, VerificationOperations};
use crate::{
    PurchaseReviewContract, PurchaseReviewContractArgs, PurchaseReviewContractClient,
    datatype::{ReviewDetails, PurchaseReviewError, DataKeys}
};
use soroban_sdk::{Env, contractimpl, Address, String, Vec};

#[contractimpl]
impl ReviewOperations for PurchaseReviewContract {
    /// Submits a new product review
    /// * `user` - Address of the user submitting the review
    /// * `product_id` - Unique identifier of the product being reviewed
    /// * `review_text` - The actual review content
    /// * `purchase_link` - Link/proof of purchase for verification
    /// Returns Result<(), PurchaseReviewError>
    fn submit_review(
        env: Env,
        user: Address,
        product_id: u128,
        review_text: String,
        purchase_link: String,
    ) -> Result<(), PurchaseReviewError> {
        user.require_auth();

        // Validate review text
        if review_text.len() == 0 || review_text.len() > 1000 {  // adjust max length as needed
            return Err(PurchaseReviewError::InvalidReviewText);
        }

        // Verify the purchase link is valid for this user and product
        Self::purchase_link_verification(env.clone(), user.clone(), product_id, purchase_link)?;

        // Check if this purchase has already been reviewed by user
        Self::pre_review_purchase(env.clone(), user.clone(), product_id)?;
        
        // Get and increment review count atomically using a transaction
        let count_key = DataKeys::ReviewCount(product_id);
        let review_id = env.storage().persistent().get(&count_key).unwrap_or(0);
        env.storage().persistent().set(&count_key, &(review_id + 1));

        // Create a new review with initial values
        let review = ReviewDetails {
            review_text,
            reviewer: user,
            timestamp: env.ledger().timestamp(),
            helpful_votes: 0,
            not_helpful_votes: 0,
            verified_purchase: true,
            responses: Vec::new(&env),
        };

        // Store the review and increment count atomically
        let key = DataKeys::Review(product_id, review_id);
        env.storage().persistent().set(&key, &review);
        
        Ok(())
    }

    /// Adds a response to an existing review
    /// * `reviewer` - Address of the user adding the response
    /// * `product_id` - ID of the product
    /// * `review_id` - ID of the review being responded to
    /// * `response_text` - The response content
    fn add_response(
        env: Env,
        reviewer: Address,
        product_id: u128,
        review_id: u32,
        response_text: String,
    ) -> Result<(), PurchaseReviewError> {
        // Verify the responder's authorization
        reviewer.require_auth();
        
        // Validate response text
        if response_text.len() == 0 || response_text.len() > 500 {
            return Err(PurchaseReviewError::InvalidResponseText);
        }

        // Retrieve the existing review
        let key = DataKeys::Review(product_id, review_id);
        let mut review = env.storage().persistent().get::<_, ReviewDetails>(&key)
            .ok_or(PurchaseReviewError::ReviewNotFound)?;
        
        
        review.responses.push_back(response_text);
        env.storage().persistent().set(&key, &review);
        Ok(())
    }

    /// Records a helpful/not helpful vote for a review
    /// * `voter` - Address of the user voting
    /// * `product_id` - ID of the product
    /// * `review_id` - ID of the review being voted on
    /// * `helpful` - true for helpful vote, false for not helpful
    fn vote_helpful(
        env: Env,
        voter: Address,
        product_id: u128,
        review_id: u32,
        helpful: bool,
    ) -> Result<(), PurchaseReviewError> {
        // Verify voter's authorization
        voter.require_auth();
        
        // Check rate limiting
        let rate_limit_key = DataKeys::VoteRateLimit(voter.clone());
        let last_vote_time = env.storage().persistent()
            .get::<_, u64>(&rate_limit_key)
            .unwrap_or(0);
        let current_time = env.ledger().timestamp();
        
        if current_time - last_vote_time < 300 { // 5 minutes cooldown
            return Err(PurchaseReviewError::RateLimitExceeded);
        }
        
        // Check if user has already voted on this review
        let vote_key = DataKeys::ReviewVote(product_id, review_id, voter.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(PurchaseReviewError::AlreadyVoted);
        }
        
        // Retrieve the review
        let key = DataKeys::Review(product_id, review_id);
        let mut review = env.storage().persistent().get::<_, ReviewDetails>(&key)
            .ok_or(PurchaseReviewError::ReviewNotFound)?;
        
        // Update the appropriate vote counter
        if helpful {
            review.helpful_votes += 1;
        } else {
            review.not_helpful_votes += 1;
        }
        
        // Record that this user has voted
        env.storage().persistent().set(&vote_key, &helpful);
        
        // Save the updated review
        env.storage().persistent().set(&key, &review);
        Ok(())
    }

    /// Verifies a purchase and adds a verification badge to the review
    /// * `user` - Address of the user requesting verification
    /// * `product_id` - ID of the product
    /// * `review_id` - ID of the review to verify
    /// * `purchase_link` - Link/proof of purchase for verification
    fn verified_purchase_badge(
        env: Env,
        user: Address,
        product_id: u128,
        review_id: u32,
        purchase_link: String,
    ) -> Result<(), PurchaseReviewError> {
        // Retrieve the review using the correct key
        let key = DataKeys::Review(product_id, review_id);
        let mut review = env.storage().persistent().get::<_, ReviewDetails>(&key)
            .ok_or(PurchaseReviewError::ReviewNotFound)?;

        // Verify the user owns this review
        if review.reviewer != user {
            return Err(PurchaseReviewError::UnauthorizedAccess);
        }

        // Verify the purchase
        Self::purchase_link_verification(env.clone(), user.clone(), product_id, purchase_link)?;
        
        // Mark the review as verified
        review.verified_purchase = true;
        
        // Save the updated review
        env.storage().persistent().set(&key, &review);
        Ok(())
    }

    /// Retrieves the details of a specific review
    /// * `product_id` - ID of the product
    /// * `review_id` - ID of the review to retrieve
    /// Returns Result<ReviewDetails, PurchaseReviewError>
    fn get_review_details(
        env: Env,
        product_id: u128,
        review_id: u32,
    ) -> Result<ReviewDetails, PurchaseReviewError> {
        let key = DataKeys::Review(product_id, review_id);
        env.storage().persistent().get::<_, ReviewDetails>(&key)
            .ok_or(PurchaseReviewError::ReviewNotFound)
    }
}