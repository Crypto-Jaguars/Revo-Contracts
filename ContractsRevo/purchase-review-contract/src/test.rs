#![cfg(test)]
use super::*;
use crate::datatype::{Category, Rating, ReviewDetails};
use crate::{PurchaseReviewContract, PurchaseReviewContractClient};
use soroban_sdk::Vec;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, String,
};

#[test]
fn test_submit_rating_events() {
    // Set up the test environment
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    // Set up the test user
    let user = Address::generate(&env);

    // Test data for the rating
    let product_id: u128 = 12345;
    let category = Category::Quality;
    let rating = Rating::FiveStars;
    let weight: u32 = 2;
    let attachment = String::from_str(&env, "Great product!");

    // Authorize the transaction
    env.mock_all_auths();

    // Submit the rating
    client.submit_rating(&user, &product_id, &category, &rating, &weight, &attachment);

    // Verify the emitted events
    let events = env.events().all();
    assert_eq!(events.len(), 2); // Expect 2 events

    // Verify the first event
    let event = events.get(0).unwrap();
    assert_eq!(event.0, contract_id);

    // Verify that the event data is not empty
    assert!(!event.2.is_void());

    // Verify the second event
    let event = events.get(1).unwrap();
    assert_eq!(event.0, contract_id);
    assert!(!event.2.is_void());

    // Verify the stored rating
    let stored_ratings = client.get_product_ratings(&product_id);
    assert_eq!(stored_ratings.ratings.len(), 1);

    // Verify the details of the stored rating
    let stored_rating = stored_ratings.ratings.get(0).unwrap();
    assert_eq!(stored_rating.category, category);
    assert_eq!(stored_rating.rating, rating);
    assert_eq!(stored_rating.user, user);
    assert_eq!(stored_rating.weight, weight * (rating as u32));
    assert_eq!(stored_rating.attachment, attachment);
}

// Helper function to set up the test environment with admin
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

#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_panic_reviewer_not_authenticated() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "This product is meh!");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_invalid_review_text_length_empty() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_invalid_review_text_length_too_long() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, &"a".repeat(1001));
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
fn test_valid_review_submission() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "This product is excellent!");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let events = env.events().all();
    assert_eq!(events.len(), 1); // Expect 1 events

    // Verify the first event
    let event = events.get(0).unwrap();
    assert_eq!(event.0, contract_id);

    // Verify that the event data is not empty
    assert!(!event.2.is_void());

    env.as_contract(&contract_id, || {
        let count_key = DataKeys::ReviewCount(product_id);
        let review_count: u32 = env
            .storage()
            .persistent()
            .get(&count_key)
            .expect("Review count not found");
        assert_eq!(review_count, 1);

        let review_key = DataKeys::Review(product_id, 0);
        let stored_review: ReviewDetails = env
            .storage()
            .persistent()
            .get(&review_key)
            .expect("Review not found");

        assert_eq!(stored_review.reviewer, user);
        assert_eq!(stored_review.review_text, review_text);
        assert_eq!(stored_review.verified_purchase, true);
        assert_eq!(stored_review.helpful_votes, 0);
        assert_eq!(stored_review.not_helpful_votes, 0);
        assert_eq!(stored_review.responses.len(), 0);
        assert_eq!(stored_review.timestamp, env.ledger().timestamp());
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #11")]
fn test_duplicate_review_submission() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "Great product!");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #22")]
fn test_invalid_purchase_link() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "Great product!");
    let purchase_link = String::from_str(&env, "");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_verify_purchase_link_already_verified() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    let product_id = 12345u128;
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.initialize(&admin);
    env.mock_all_auths();
    client.purchase_link_verification(&user, &product_id, &purchase_link);
    client.purchase_link_verification(&user, &product_id, &purchase_link);
}

#[test]
fn test_verify_purchase_link() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    let product_id = 12345u128;
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.initialize(&admin);
    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_report_invalid_review() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "Initial review text.");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let reporter = Address::generate(&env);
    let reason = String::from_str(&env, "Inappropriate content.");

    client.report_review(&reporter, &product_id, &10, &reason);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_report_review_with_empty_reason() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "Initial review text.");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let reporter = Address::generate(&env);
    let reason = String::from_str(&env, "");

    client.report_review(&reporter, &product_id, &0, &reason);
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")]
fn test_report_review_already_reported() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "Initial review text.");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let reporter = Address::generate(&env);
    let reason = String::from_str(&env, "Inappropriate content.");

    client.report_review(&reporter, &product_id, &0, &reason);
    client.report_review(&reporter, &product_id, &0, &reason);
}

#[test]
fn test_report_review_for_inappropriate_content() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "Initial review text.");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let reporter = Address::generate(&env);
    let reason = String::from_str(&env, "Inappropriate content.");

    client.report_review(&reporter, &product_id, &0, &reason);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_rate_limit_exceeded_for_voting() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "Initial review text.");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let voter = Address::generate(&env);
    client.vote_helpful(&voter, &product_id, &0, &true);

    // Attempt to vote again within the cooldown period
    client.vote_helpful(&voter, &product_id, &0, &true);
}
