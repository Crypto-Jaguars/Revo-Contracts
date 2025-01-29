use crate::interface::VerificationOperations;
use crate::{
    PurchaseReviewContract, PurchaseReviewContractClient, PurchaseReviewContractArgs,
    datatype::{PurchaseReviewError, ReviewDetails, DataKeys, PurchaseVerificationData, ReviewReportData}
};
use soroban_sdk::{Env, contractimpl, Address, String};

/// Time window (in seconds) during which a review can be edited (24 hours). 
/// Cross check this with the timestamp when testing or on mainnet.
const REVIEW_EDIT_WINDOW: u64 = 24 * 60 * 60; 


#[contractimpl]
impl VerificationOperations for PurchaseReviewContract {
    /// Internal functiion that will be ccalled in review..rs when submitting review.
    /// Checks if a user has already reviewed a purchase and marks it as reviewed if not
    /// 
    /// # Arguments
    /// * `user` - Address of the user
    /// * `purchase_id` - Unique identifier of the purchase
    /// 
    /// # Returns
    /// * `Result<bool, PurchaseReviewError>` - True if the purchase has been reviewed
    fn pre_review_purchase(
        env: Env,
        user: Address,
        product_id: u128,
    ) -> Result<bool, PurchaseReviewError> {
        // Use product_id as the key since it's unique per purchase
        let key = DataKeys::PurchaseVerification(product_id, user.clone());
        let mut purchase_data = env.storage().persistent().get::<_, PurchaseVerificationData>(&key)
            .ok_or(PurchaseReviewError::PurchaseNotFound)?;

        // Verify the user matches
        if purchase_data.user != user {
            return Err(PurchaseReviewError::UnauthorizedAccess);
        }

        if purchase_data.has_review {
            return Err(PurchaseReviewError::AlreadyReviewed);
        }

        purchase_data.has_review = true;
        env.storage().persistent().set(&key, &purchase_data);

        Ok(purchase_data.has_review)
    }


    /// Reports a review for inappropriate content or other issues
    /// 
    /// # Arguments
    /// * `reporter` - Address of the user reporting the review
    /// * `product_id` - Unique identifier of the product
    /// * `review_id` - Unique identifier of the review
    /// * `reason` - Description of why the review is being reported
    /// 
    /// # Returns
    /// * `Result<(), PurchaseReviewError>` - Success or error status
    fn report_review(
        env: Env,
        reporter: Address,
        product_id: u128,
        review_id: u32,
        reason: String,
    ) -> Result<(), PurchaseReviewError> {
        // Require authentication from the reporter
        reporter.require_auth();

        // Validate reason text length
        if reason.len() == 0 || reason.len() > 500 {
            return Err(PurchaseReviewError::InvalidReportReason);
        }

        // Check if user has already reported this review
        let user_report_key = DataKeys::UserReviewReport(product_id, review_id, reporter.clone());
        if env.storage().persistent().has(&user_report_key) {
            return Err(PurchaseReviewError::AlreadyReported);
        }

        // Verify review exists
        let review_key = DataKeys::Review(product_id, review_id);
        env.storage().persistent().get::<_, ReviewDetails>(&review_key)
            .ok_or(PurchaseReviewError::ReviewNotFound)?;

        // Create and store report data
        let report = ReviewReportData {
            reporter,
            product_id,
            review_id,
            reason,
            timestamp: env.ledger().timestamp(),
        };

        let report_key = DataKeys::ReviewReport(product_id, review_id);
        env.storage().persistent().set(&report_key, &report);
        // Mark that this user has reported this review
        env.storage().persistent().set(&user_report_key, &true);
        Ok(())
    }


    /// Internal function that will be called in review.rs file
    /// Verifies a purchase by linking it with proof of purchase
    /// 
    /// # Arguments
    /// * `user` - Address of the user who made the purchase
    /// * `purchase_id` - Unique identifier of the purchase
    /// * `purchase_link` - Proof of purchase link or identifier
    /// 
    /// # Returns
    /// * `Result<(), PurchaseReviewError>` - Success or error status
    fn purchase_link_verification(
        env: Env,
        user: Address,
        product_id: u128,
        purchase_link: String,
    ) -> Result<(), PurchaseReviewError> {
        user.require_auth();
        // Create a composite key using both product_id and user address
        let key = DataKeys::PurchaseVerification(product_id, user.clone());
        
        if env.storage().persistent().has(&key) {
            return Err(PurchaseReviewError::AlreadyVerified);
        }

        let verification_data = PurchaseVerificationData {
            user,
            product_id,
            purchase_link,
            is_verified: true,
            timestamp: env.ledger().timestamp(),
            has_review: false,
        };
        env.storage().persistent().set(&key, &verification_data);
        Ok(())
    }


    /// Edits an existing review if within the edit window
    /// 
    /// # Arguments
    /// * `user` - Address of the user editing the review
    /// * `product_id` - Unique identifier of the product
    /// * `review_id` - Unique identifier of the review
    /// * `new_details` - Updated review details
    /// 
    /// # Returns
    /// * `Result<(), PurchaseReviewError>` - Success or error status
    fn edit_review(
        env: Env,
        user: Address,
        product_id: u128,
        review_id: u32,
        new_details: ReviewDetails,
    ) -> Result<(), PurchaseReviewError> {
        // Require authentication from the user
        user.require_auth();

        // Check if review is still editable
        if !Self::is_review_editable(env.clone(), review_id, product_id)? {
            return Err(PurchaseReviewError::EditWindowExpired);
        }

        // Get the existing review
        let key = DataKeys::Review(product_id, review_id);
        let existing_review = env.storage().persistent().get::<_, ReviewDetails>(&key)
            .ok_or(PurchaseReviewError::ReviewNotFound)?;

        // Verify the user is the original reviewer
        if existing_review.reviewer != user {
            return Err(PurchaseReviewError::UnauthorizedAccess);
        }

        // Update the review while preserving original timestamp and reviewer
        let updated_review = ReviewDetails {
            timestamp: existing_review.timestamp, // Preserve original timestamp
            reviewer: existing_review.reviewer,   // Preserve original reviewer
            ..new_details // Update all other fields from new_details
        };

        // Store the updated review
        env.storage().persistent().set(&key, &updated_review);
        Ok(())
    }


    /// Internal function to check if a review is still within the editable time window
    /// 
    /// # Arguments
    /// * `review_id` - Unique identifier of the review
    /// * `product_id` - Unique identifier of the product
    /// 
    /// # Returns
    /// * `Result<bool, PurchaseReviewError>` - True if review is still editable
    fn is_review_editable(
        env: Env,
        review_id: u32,
        product_id: u128,
    ) -> Result<bool, PurchaseReviewError> {
        // Retrieve review details
        let key = DataKeys::Review(product_id, review_id);
        let review = env.storage().persistent().get::<_, ReviewDetails>(&key)
            .ok_or(PurchaseReviewError::ReviewNotFound)?;

        // Calculate time elapsed since review creation
        let current_time = env.ledger().timestamp();
        let time_elapsed = current_time - review.timestamp;

        Ok(time_elapsed <= REVIEW_EDIT_WINDOW)
    }
}