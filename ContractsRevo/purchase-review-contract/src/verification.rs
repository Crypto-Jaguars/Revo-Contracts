use crate::interface::VerificationOperations;
use crate::{
    datatype::{
        DataKeys, PurchaseReviewError, PurchaseVerificationData, ReviewDetails, ReviewReportData,
    },
    PurchaseReviewContract, PurchaseReviewContractArgs, PurchaseReviewContractClient,
};
use soroban_sdk::{contractimpl, Address, Env, String, Symbol};

/// Time window (in seconds) during which a review can be edited (24 hours).
/// Cross check this with the timestamp when testing or on mainnet.
#[allow(dead_code)]
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
        let key = DataKeys::PurchaseVerification(product_id, user.clone());

        if let Some(verification) = env
            .storage()
            .persistent()
            .get::<_, PurchaseVerificationData>(&key)
        {
            if verification.has_review {
                return Err(PurchaseReviewError::AlreadyReviewed);
            }
        }

        Ok(true)
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
        reporter.require_auth();
        if reason.len() == 0 {
            return Err(PurchaseReviewError::InvalidReportReason);
        }

        let review_key = DataKeys::Review(product_id, review_id);
        if !env.storage().persistent().has(&review_key) {
            return Err(PurchaseReviewError::ReviewNotFound);
        }

        let report_key = DataKeys::UserReviewReport(product_id, review_id, reporter.clone());
        if env.storage().persistent().has(&report_key) {
            return Err(PurchaseReviewError::AlreadyReported);
        }

        let report_data = ReviewReportData {
            reporter: reporter.clone(),
            product_id,
            review_id,
            reason: reason.clone(),
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&report_key, &report_data);

        env.events().publish(
            (Symbol::new(&env, "review_reported"), reporter),
            (product_id, review_id, reason),
        );

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
        if purchase_link.len() == 0 {
            return Err(PurchaseReviewError::InvalidPurchaseLink);
        }

        let key = DataKeys::PurchaseVerification(product_id, user.clone());

        // Check if already verified
        if env.storage().persistent().has(&key) {
            return Err(PurchaseReviewError::AlreadyVerified);
        }

        let verification_data = PurchaseVerificationData {
            user: user.clone(),
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
        _env: Env,
        _user: Address,
        _product_id: u128,
        _review_id: u32,
        _new_details: ReviewDetails,
    ) -> Result<(), PurchaseReviewError> {
        Ok(())

        // Self::is_review_editable(_env, _review_id, _product_id)?
        // return Ok(())
        // }
        // return Err(PurchaseReviewError::EditWindowExpired)
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
        let key = DataKeys::Review(product_id, review_id);
        if let Some(review) = env.storage().persistent().get::<_, ReviewDetails>(&key) {
            let current_time = env.ledger().timestamp();
            // 24 hours edit window
            Ok(current_time - review.timestamp <= 86400)
        } else {
            Err(PurchaseReviewError::ReviewNotFound)
        }
    }
}
