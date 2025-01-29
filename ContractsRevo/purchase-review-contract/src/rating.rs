use crate::interface::RatingOperations;
use crate::{
PurchaseReviewContract, PurchaseReviewContractArgs, PurchaseReviewContractClient,
datatype::{Category, Rating, ProductRatings, CategoryRating, PurchaseReviewError, DataKeys}
};
use soroban_sdk::{Env, contractimpl, Address, Vec, Symbol, String};



#[contractimpl]
impl RatingOperations for PurchaseReviewContract {
    /// Submits a new product rating with category, weight, and attachments
    /// Parameters:
    /// - env: The Soroban environment
    /// - user: Address of the user submitting the rating
    /// - product_id: Unique identifier for the product
    /// - category: Product category being rated
    /// - rating: The actual rating value
    /// - weight: Importance factor for this rating
    /// - attachment: Additional comments or feedback as a string
    fn submit_rating(
        env: Env,
        user: Address,
        product_id: u128,
        category: Category,
        rating: Rating,
        weight: u32,
        attachment: String
    ) -> Result<(), PurchaseReviewError> {
        // Verify that the transaction is signed by the user submitting the rating
        // This prevents unauthorized submissions
        user.require_auth();

        // Create a storage key for the product's ratings using the product_id
        // This key is used to store/retrieve ratings in the contract's persistent storage
        let key = DataKeys::ProductRatings(product_id);

        // Retrieve existing ratings for the product from storage
        // If no ratings exist yet, create a new empty ratings collection
        let mut product_ratings = env.storage().persistent().get::<_, ProductRatings>(&key)
            .unwrap_or_else(|| ProductRatings { ratings: Vec::new(&env) });
 
        // Calculate the weighted rating value based on the rating and weight factors
        let weighted_rating = Self::calculate_weighted(&env, rating.clone(), weight)?;

        // Create a new rating entry with all the provided details
        // timestamp is automatically set to the current ledger time
        let category_rating = CategoryRating {
            category,
            rating,
            timestamp: env.ledger().timestamp(), 
            attachment,
            user,
            weight: weighted_rating
        };
 
        // Add the new rating to the product's ratings collection
        product_ratings.ratings.push_back(category_rating);

        // Save the updated ratings back to persistent storage
        // This ensures the new rating is permanently stored on the blockchain
        env.storage().persistent().set(&key, &product_ratings);
        Ok(())
    }

    /// Internal function that calculates the final rating value adjusted by weight factor
    /// Parameters:
    /// - env: Reference to the Soroban environment
    /// - rating: The base rating value
    /// - weight: The weight factor to apply
    /// Returns: The weighted rating value as u32
    fn calculate_weighted(env: &Env, rating: Rating, weight: u32) -> Result<u32, PurchaseReviewError> {
        // Convert rating enum to numeric value
        let rating_value = rating as u32;

        // Multiply rating by weight, handling potential overflow
        let weighted_rating = rating_value.checked_mul(weight)
            .ok_or(PurchaseReviewError::WeightedRatingOverflow)?;

        // Emit an event with the calculation details for transparency
        env.events().publish(
            (
                Symbol::new(env, "calculated_weighted_rating"),
                rating_value,
            ),
            weighted_rating,
        );

        Ok(weighted_rating)
    }

    /// Retrieves all ratings for a given product ID
    /// Parameters:
    /// - env: The Soroban environment
    /// - product_id: The unique identifier of the product
    /// Returns: Result containing ProductRatings or an error if product not found
    fn get_product_ratings(
        env: Env,
        product_id: u128
    ) -> Result<ProductRatings, PurchaseReviewError> {
        // Create a storage key for the product's ratings
        let key = DataKeys::ProductRatings(product_id);

        // Retrieve ratings from storage
        // Returns empty ratings collection if none exist yet
        let product_ratings = env.storage().persistent().get::<_, ProductRatings>(&key)
            .unwrap_or_else(|| ProductRatings { ratings: Vec::new(&env) });

        Ok(product_ratings)
    }
}