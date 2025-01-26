use crate::interface::RatingOperations;
use crate::{
PurchaseReviewContract, PurchaseReviewContractArgs, PurchaseReviewContractClient,
datatype::{Category, Rating, Attachment, ProductRatings, CategoryRating, PurchaseReviewError, DataKeys}
};
use soroban_sdk::{Env, contractimpl, Address, Vec, Symbol};



#[contractimpl]
impl RatingOperations for PurchaseReviewContract {
    /// Submits a new product rating with category, weight, and attachments
    fn submit_rating(
        env: Env,
        user: Address,
        product_id: u128,
        category: Category,
        rating: Rating,
        weight: u32,
        attachment: Attachment
    ) -> Result<(), PurchaseReviewError> {
        // Verify that the transaction is signed by the user submitting the rating
        user.require_auth();

        // Create a storage key for the product's ratings using the product_id
        let key = DataKeys::ProductRatings(product_id);

        // Retrieve existing ratings for the product from storage, or create new empty ratings if none exist
        let mut product_ratings = env.storage().persistent().get::<_, ProductRatings>(&key)
            .unwrap_or_else(|| ProductRatings { ratings: Vec::new(&env) });
 
        let weighted_rating = Self::calculate_weighted(&env, rating.clone(), weight);
        // Create a new rating entry with the provided details and current timestamp
        let category_rating = CategoryRating {
            category,
            rating,
            timestamp: env.ledger().timestamp(), 
            attachments: attachment,
            user,
            weight: weighted_rating as u32
        };
 
        // Add the new rating to the product's ratings collection
        product_ratings.ratings.push_back(category_rating);
        // Save the updated ratings back to persistent storage
        env.storage().persistent().set(&key, &product_ratings);
        Ok(())
    }

    /// Calculates rating value adjusted by weight factor
    fn calculate_weighted(env: &Env, rating: Rating, weight: u32) -> u32 {
        let rating_value = rating as u32;
        let weighted_rating = rating_value * weight;

        env.events().publish(
            (
                Symbol::new(env, "calculated_weighted_rating"),
                rating_value,
            ),
            weighted_rating,
        );

        weighted_rating
    }


    /// Retrieves all ratings for a given product ID
    fn get_product_ratings(
        env: Env,
        product_id: u128
    ) -> Result<ProductRatings, PurchaseReviewError> {
        // Create a storage key for the product's ratings
        let key = DataKeys::ProductRatings(product_id);

        // Retrieve ratings from storage
        let product_ratings = env.storage().persistent().get::<_, ProductRatings>(&key)
            .ok_or(PurchaseReviewError::ProductNotFound)?;

        Ok(product_ratings)
    }
}