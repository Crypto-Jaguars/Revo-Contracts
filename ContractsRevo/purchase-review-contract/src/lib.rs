#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Symbol};

use crate::datatype::{
    DataKeys, ProductRatings, PurchaseReviewError, PurchaseVerificationData, ReviewDetails,
};

mod datatype;
mod interface;
mod rating;
mod review;
mod verification;

#[cfg(test)]
mod test;

#[cfg(test)]
mod tests;

#[contract]
pub struct PurchaseReviewContract;

#[contractimpl]
impl PurchaseReviewContract {
    // Initialize the contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), PurchaseReviewError> {
        if env.storage().instance().has(&DataKeys::Admin) {
            return Err(PurchaseReviewError::AlreadyVerified);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKeys::Admin, &admin);

        env.events().publish(
            (Symbol::new(&env, "contract_initialized"), admin.clone()),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, PurchaseReviewError> {
        env.storage()
            .instance()
            .get(&DataKeys::Admin)
            .ok_or(PurchaseReviewError::UnauthorizedAccess)
    }

    pub fn verify_purchase(
        env: Env,
        user: Address,
        product_id: u64,
        purchase_link: String,
    ) -> Result<bool, PurchaseReviewError> {
        let admin = Self::get_admin(env.clone())?;
        admin.require_auth();

        let verification_data = PurchaseVerificationData {
            user: user.clone(),
            product_id,
            purchase_link: purchase_link.clone(),
            is_verified: true,
            timestamp: env.ledger().timestamp(),
            has_review: false,
        };

        let key = DataKeys::PurchaseVerification(product_id, user.clone());
        env.storage().persistent().set(&key, &verification_data);

        env.events().publish(
            (Symbol::new(&env, "purchase_verified"), user.clone()),
            product_id,
        );

        Ok(true)
    }

    pub fn is_purchase_verified(
        env: Env,
        _user: Address,
        product_id: u64,
    ) -> Result<bool, PurchaseReviewError> {
        let verification_data = env
            .storage()
            .persistent()
            .get::<_, PurchaseVerificationData>(&DataKeys::PurchaseVerification(product_id, _user))
            .ok_or(PurchaseReviewError::PurchaseNotFound)?;

        Ok(verification_data.is_verified)
    }

    pub fn get_product_rating(
        env: Env,
        product_id: u64,
    ) -> Result<(u32, u32), PurchaseReviewError> {
        let mut total_rating = 0u32;
        let mut total_reviews = 0u32;

        let reviews_key = DataKeys::ProductRatings(product_id);
        if let Some(ratings) = env
            .storage()
            .persistent()
            .get::<_, ProductRatings>(&reviews_key)
        {
            for rating in ratings.ratings.iter() {
                total_rating += rating.rating as u32;
                total_reviews += 1;
            }
        }

        if total_reviews == 0 {
            return Ok((0, 0));
        }

        Ok((total_rating / total_reviews, total_reviews))
    }

    pub fn get_review(
        env: Env,
        product_id: u64,
        review_id: u32,
    ) -> Result<ReviewDetails, PurchaseReviewError> {
        let key = DataKeys::Review(product_id, review_id);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(PurchaseReviewError::ReviewNotFound)
    }
}
