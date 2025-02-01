use crate::datatype::{Category, ProductRatings, PurchaseReviewError, Rating, ReviewDetails};
use soroban_sdk::{Address, Env, String};

/// Handles rating-related operations for products
#[allow(dead_code)]
pub trait RatingOperations {
    /// Submits a new rating for a product with optional attachments
    /// Returns an error if the submission fails
    fn submit_rating(
        env: Env,
        user: Address,
        product_id: u128,
        category: Category,
        rating: Rating,
        weight: u32,
        attachment: String,
    ) -> Result<(), PurchaseReviewError>;

    /// Calculates the weighted rating score based on rating value and weight
    fn calculate_weighted(
        env: &Env,
        rating: Rating,
        weight: u32,
    ) -> Result<u32, PurchaseReviewError>;

    /// Retrieves all ratings for a specific product
    /// Returns ProductRatings containing aggregate rating data
    fn get_product_ratings(
        env: Env,
        product_id: u128,
    ) -> Result<ProductRatings, PurchaseReviewError>;
}

/// Manages review-related operations including submissions, responses, and voting
#[allow(dead_code)]
pub trait ReviewOperations {
    /// Submits a new review with purchase verification
    /// purchase_link serves as proof of purchase
    fn submit_review(
        env: Env,
        user: Address,
        product_id: u128,
        review_text: String,
        purchase_link: String, // Transaction/order ID
    ) -> Result<(), PurchaseReviewError>;

    /// Allows adding responses to existing reviews
    /// Typically used for merchant/seller responses
    fn add_response(
        env: Env,
        reviewer: Address,
        product_id: u128,
        review_id: u32,
        response_text: String,
    ) -> Result<(), PurchaseReviewError>;

    /// Enables users to vote on review helpfulness
    /// Helps in ranking and displaying most helpful reviews
    fn vote_helpful(
        env: Env,
        voter: Address,
        product_id: u128,
        review_id: u32,
        helpful: bool,
    ) -> Result<(), PurchaseReviewError>;

    /// Assigns a verified purchase badge based on valid purchase proof
    fn verified_purchase_badge(
        env: Env,
        user: Address,
        product_id: u128,
        review_id: u32,
        purchase_link: String,
    ) -> Result<(), PurchaseReviewError>;

    /// Retrieves detailed information about a specific review
    /// Including verification status and helpfulness votes
    fn get_review_details(
        env: Env,
        product_id: u128,
        review_id: u32,
    ) -> Result<ReviewDetails, PurchaseReviewError>;
}

/// Handles verification and moderation-related operations
#[allow(dead_code)]
pub trait VerificationOperations {
    /// Checks if user has already submitted a review for this purchase
    /// Prevents duplicate reviews
    fn pre_review_purchase(
        env: Env,
        user: Address,
        product_id: u128,
    ) -> Result<bool, PurchaseReviewError>;

    /// Verifies if a review is still within the editable timeframe
    /// Reviews may be locked after a certain period
    fn is_review_editable(
        env: Env,
        review_id: u32,
        product_id: u128,
    ) -> Result<bool, PurchaseReviewError>;

    /// Allows users to report reviews for moderation
    /// Helps maintain review quality and prevent abuse
    fn report_review(
        env: Env,
        reporter: Address,
        product_id: u128,
        review_id: u32,
        reason: String,
    ) -> Result<(), PurchaseReviewError>;

    /// Verifies the authenticity of a purchase using the provided purchase link
    fn purchase_link_verification(
        env: Env,
        user: Address,
        product_id: u128,
        purchase_link: String,
    ) -> Result<(), PurchaseReviewError>;

    /// Allows users to edit their reviews within the editable timeframe
    fn edit_review(
        env: Env,
        user: Address,
        product_id: u128,
        review_id: u32,
        new_details: ReviewDetails,
    ) -> Result<(), PurchaseReviewError>;
}
