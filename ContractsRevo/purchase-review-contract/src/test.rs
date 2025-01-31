#![cfg(test)]
use super::*;
use crate::datatype::{PurchaseReviewError, ReviewDetails};
// use crate::interface::ReviewOperations;
use crate::datatype::Category;
use crate::datatype::Rating;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};
use soroban_sdk::{String, Vec};

// Helper function to setup test environment with admin
fn setup_test() -> (Env, PurchaseReviewContractClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Mock auths before initialization
    env.mock_all_auths();
    client.initialize(&admin);

    (env, client, admin, user)
}

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // First initialization should succeed
    env.mock_all_auths();
    client.initialize(&admin);

    // Second initialization should fail
    env.mock_all_auths();
    let result = client.try_initialize(&admin);
    assert!(result.is_err());

    // Verify admin was set correctly
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_verify_purchase() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;
    let purchase_link = String::from_str(&env, "https://valid-purchase.com/123");

    // Test purchase verification by admin
    env.mock_all_auths();
    assert!(client.verify_purchase(&user, &product_id, &purchase_link));

    // Verify the purchase status
    assert!(client.is_purchase_verified(&user, &product_id));
}

#[test]
fn test_get_product_rating() {
    let (_, client, _, _) = setup_test();
    let product_id = 1u128;

    // Test empty product rating
    let (avg_rating, total_reviews) = client.get_product_rating(&product_id);
    assert_eq!(avg_rating, 0);
    assert_eq!(total_reviews, 0);
}

#[test]
fn test_get_review() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;
    let review_id = 0u32;

    // Create review details
    let review = ReviewDetails {
        review_text: String::from_str(&env, "Great product!"),
        reviewer: user.clone(),
        timestamp: env.ledger().timestamp(),
        helpful_votes: 0,
        not_helpful_votes: 0,
        verified_purchase: true,
        responses: Vec::new(&env),
    };

    // Store the review in storage within contract context
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&(product_id, review_id), &review);
    });

    // Retrieve and verify the review
    let retrieved_review = client.get_review(&product_id, &review_id);
    assert_eq!(
        retrieved_review.review_text,
        String::from_str(&env, "Great product!")
    );
    assert_eq!(retrieved_review.reviewer, user);
    assert_eq!(retrieved_review.helpful_votes, 0);
    assert_eq!(retrieved_review.not_helpful_votes, 0);
    assert_eq!(retrieved_review.verified_purchase, true);
    assert_eq!(retrieved_review.responses.len(), 0);
}

#[test]
fn test_submit_rating() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;

    env.mock_all_auths();

    // Submit a rating
    client.submit_rating(
        &user,
        &product_id,
        &Category::Quality,
        &Rating::FourStars,
        &2,
        &String::from_str(&env, "Great quality product!"),
    );

    // Verify the rating was stored
    let ratings = client.get_product_ratings(&product_id);
    assert_eq!(ratings.ratings.len(), 1);

    let rating = ratings.ratings.get(0).unwrap();
    assert_eq!(rating.category, Category::Quality);
    assert_eq!(rating.rating as u32, Rating::FourStars as u32);
    assert_eq!(rating.user, user);
    assert_eq!(rating.weight, 8); // 4 stars * weight of 2
}

#[test]
fn test_submit_multiple_category_ratings() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;

    env.mock_all_auths();

    // Submit ratings for different categories
    client.submit_rating(
        &user,
        &product_id,
        &Category::Quality,
        &Rating::FiveStars,
        &1,
        &String::from_str(&env, "Excellent quality!"),
    );

    client.submit_rating(
        &user,
        &product_id,
        &Category::Shipping,
        &Rating::FourStars,
        &1,
        &String::from_str(&env, "Fast shipping"),
    );

    client.submit_rating(
        &user,
        &product_id,
        &Category::CustomerService,
        &Rating::ThreeStars,
        &1,
        &String::from_str(&env, "Average service"),
    );

    let ratings = client.get_product_ratings(&product_id);
    assert_eq!(ratings.ratings.len(), 3);
}

#[test]
fn test_weighted_rating_calculation() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;

    env.mock_all_auths();

    // Test different weights
    client.submit_rating(
        &user,
        &product_id,
        &Category::Quality,
        &Rating::FiveStars,
        &2,
        &String::from_str(&env, "Weighted review"),
    );

    let ratings = client.get_product_ratings(&product_id);
    let rating = ratings.ratings.get(0).unwrap();
    assert_eq!(rating.weight, 10); // 5 stars * weight of 2
}

#[test]
fn test_report_review() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;
    let review_id = 0u32;

    // First create a review to report
    let review = ReviewDetails {
        review_text: String::from_str(&env, "Test review"),
        reviewer: user.clone(),
        timestamp: env.ledger().timestamp(),
        helpful_votes: 0,
        not_helpful_votes: 0,
        verified_purchase: true,
        responses: Vec::new(&env),
    };

    // Store the review using the correct DataKeys format
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKeys::Review(product_id, review_id), &review);
    });

    // Report the review
    let reporter = Address::generate(&env);
    env.mock_all_auths();

    // First report should succeed
    let result = client.try_report_review(
        &reporter,
        &product_id,
        &review_id,
        &String::from_str(&env, "Inappropriate content"),
    );
    assert!(result.is_ok());

    // Second report should fail
    env.mock_all_auths();
    let result = client.try_report_review(
        &reporter,
        &product_id,
        &review_id,
        &String::from_str(&env, "Inappropriate content"),
    );
    assert!(result.is_err());
}

#[test]
fn test_review_edit_window() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;
    let review_id = 0u32;

    // Create initial review
    let review = ReviewDetails {
        review_text: String::from_str(&env, "Initial review"),
        reviewer: user.clone(),
        timestamp: env.ledger().timestamp(),
        helpful_votes: 0,
        not_helpful_votes: 0,
        verified_purchase: true,
        responses: Vec::new(&env),
    };

    // Store the review
    env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .set(&DataKeys::Review(product_id, review_id), &review);
    });

    // Check if review is editable within window
    assert!(client.is_review_editable(&review_id, &product_id));

    // Advance time beyond edit window (24 hours + 1 second)
    env.ledger().set_timestamp(env.ledger().timestamp() + 86401);

    // Check if review is no longer editable
    assert!(!client.is_review_editable(&review_id, &product_id));
}

#[test]
fn test_pre_review_purchase() {
    let (env, client, _, user) = setup_test();
    let product_id = 1u128;

    // Verify purchase first
    env.mock_all_auths();
    client.verify_purchase(
        &user,
        &product_id,
        &String::from_str(&env, "https://valid-purchase.com/123"),
    );

    // Test pre-review check
    let result = client.pre_review_purchase(&user, &product_id);
    assert!(result);

    // Simulate a review being submitted
    let verification_data = PurchaseVerificationData {
        user: user.clone(),
        product_id,
        purchase_link: String::from_str(&env, "https://valid-purchase.com/123"),
        is_verified: true,
        timestamp: env.ledger().timestamp(),
        has_review: true,
    };

    env.as_contract(&client.address, || {
        env.storage().persistent().set(
            &DataKeys::PurchaseVerification(product_id, user.clone()),
            &verification_data,
        );
    });

    // Try pre-review check again - should fail
    let result = client.try_pre_review_purchase(&user, &product_id);
    assert!(result.is_err());
}

#[test]
fn test_get_product_ratings_empty() {
    let (_, client, _, _) = setup_test();
    let product_id = 999u128; // Using a product ID that hasn't been rated

    let ratings = client.get_product_ratings(&product_id);
    assert_eq!(ratings.ratings.len(), 0);
}
