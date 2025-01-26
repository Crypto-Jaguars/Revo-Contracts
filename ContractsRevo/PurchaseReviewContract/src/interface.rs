use soroban_sdk::{Address, Env};
use crate::datatype::{
   Category, Rating, Attachment, ProductRatings, 
   PurchaseReviewError
};

pub trait RatingOperations {
   // Submit a new rating with optional attachments
   fn submit_rating(
       env: Env,
       user: Address,
       product_id: u128,
       category: Category,
       rating: Rating,
       weight: u32,
       attachment: Attachment
   ) -> Result<(), PurchaseReviewError>;

   fn calculate_weighted(env: &Env, rating: Rating, weight: u32) -> u32;

   fn get_product_ratings(
      env: Env,
      product_id: u128
  ) -> Result<ProductRatings, PurchaseReviewError>;
}