use crate::interface::RatingOperations;
use crate::{
    PurchaseReviewContract, PurchaseReviewContractArgs, PurchaseReviewContractClient,
    datatype::{
        Category, Rating, ProductRatings, PurchaseReviewError,
        DataKeys, CategoryRating
    }
};
use soroban_sdk::{Env, contractimpl, Address, String, Vec, Symbol};

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
        user.require_auth();

        let key = DataKeys::ProductRatings(product_id);
        
        let mut product_ratings = env.storage().persistent().get::<_, ProductRatings>(&key)
            .unwrap_or_else(|| ProductRatings { ratings: Vec::new(&env) });

        let weighted_rating = Self::calculate_weighted(&env, rating.clone(), weight)?;

        let category_rating = CategoryRating {
            category,
            rating,
            timestamp: env.ledger().timestamp(),
            attachment,
            user: user.clone(),
            weight: weighted_rating
        };

        product_ratings.ratings.push_back(category_rating);
        env.storage().persistent().set(&key, &product_ratings);

        env.events().publish(
            (Symbol::new(&env, "rating_submitted"), user),
            (product_id, rating as u32, weighted_rating)
        );

        Ok(())
    }

    /// Internal function that calculates the final rating value adjusted by weight factor
    /// Parameters:
    /// - env: Reference to the Soroban environment
    /// - rating: The base rating value
    /// - weight: The weight factor to apply
    /// Returns: The weighted rating value as u32
    fn calculate_weighted(env: &Env, rating: Rating, weight: u32) -> Result<u32, PurchaseReviewError> {
        let rating_value = rating as u32;
        let weighted_rating = rating_value.checked_mul(weight)
            .ok_or(PurchaseReviewError::WeightedRatingOverflow)?;

        env.events().publish(
            (Symbol::new(env, "weighted_rating_calculated"), rating_value),
            weighted_rating
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
        let key = DataKeys::ProductRatings(product_id);
        
        let product_ratings = env.storage().persistent().get::<_, ProductRatings>(&key)
            .unwrap_or_else(|| ProductRatings { ratings: Vec::new(&env) });

        env.events().publish(
            (Symbol::new(&env, "ratings_retrieved"), product_id),
            product_ratings.ratings.len()
        );

        Ok(product_ratings)
    }
}