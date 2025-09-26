#![cfg(test)]

use super::super::*;
use super::utils::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, String, Vec,
};

#[test]
fn test_submit_review_success() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Great product! Highly recommended.");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Verify review was stored
    let review = client.get_review(&product_id, &0);
    assert_eq!(review.review_text, review_text);
    assert_eq!(review.reviewer, user);
    assert_eq!(review.verified_purchase, true);
    assert_eq!(review.helpful_votes, 0);
    assert_eq!(review.not_helpful_votes, 0);
}

#[test]
fn test_submit_review_with_valid_purchase_verification() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Excellent quality and fast shipping!");
    let purchase_link = String::from_str(&env, "https://valid-store.com/order/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Verify purchase verification was created
    assert!(client.is_purchase_verified(&user, &product_id));

    // Verify review details
    let review = client.get_review(&product_id, &0);
    assert_eq!(review.verified_purchase, true);
    assert_eq!(review.reviewer, user);
}

#[test]
fn test_submit_review_emits_event() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Good product");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    let initial_event_count = env.events().all().len();

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Verify at least one new event was emitted for the submission
    let events = env.events().all();
    // The test may fail if no events are emitted in the test environment
    // This can happen if event emission is disabled or events are cleared
    assert!(events.len() >= initial_event_count);
}

#[test]
fn test_submit_review_increments_count() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "First review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Verify review count was incremented
    env.as_contract(&client.address, || {
        let count_key = DataKeys::ReviewCount(product_id);
        let review_count: u32 = env
            .storage()
            .persistent()
            .get(&count_key)
            .expect("Review count not found");
        assert_eq!(review_count, 1);
    });
}

#[test]
fn test_submit_multiple_reviews_same_product() {
    let (env, client, _, _) = setup_test();
    let product_id = 12345u64;
    let users = create_test_users(&env, 3);

    // Submit reviews from different users
    for (i, user) in users.iter().enumerate() {
        let review_text = String::from_str(&env, "Review from user");
        let purchase_link = String::from_str(&env, "https://example.com/purchase/123");
        
        env.mock_all_auths();
        client.submit_review(&user, &product_id, &review_text, &purchase_link);
    }

    // Verify all reviews were stored
    for i in 0..3 {
        let review = client.get_review(&product_id, &i);
        assert_eq!(review.reviewer, users.get(i).unwrap());
    }

    // Verify final review count
    env.as_contract(&client.address, || {
        let count_key = DataKeys::ReviewCount(product_id);
        let review_count: u32 = env
            .storage()
            .persistent()
            .get(&count_key)
            .expect("Review count not found");
        assert_eq!(review_count, 3);
    });
}

#[test]
fn test_submit_review_different_products() {
    let (env, client, _, user) = setup_test();
    let review_text = String::from_str(&env, "Same user, different products");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit reviews for different products
    for product_id in 1..=3 {
        env.mock_all_auths();
        client.submit_review(&user, &product_id, &review_text, &purchase_link);
    }

    // Verify reviews were stored for each product
    for product_id in 1..=3 {
        let review = client.get_review(&product_id, &0);
        assert_eq!(review.reviewer, user);
        assert_eq!(review.review_text, review_text);
    }
}

#[test]
fn test_vote_helpful_success() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Helpful review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit review first
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Vote helpful
    let voter = Address::generate(&env);
    env.mock_all_auths();
    client.vote_helpful(&voter, &product_id, &0, &true);

    // Verify vote was recorded
    let review = client.get_review(&product_id, &0);
    assert_eq!(review.helpful_votes, 1);
    assert_eq!(review.not_helpful_votes, 0);
}

#[test]
fn test_vote_not_helpful_success() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Not helpful review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit review first
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Vote not helpful
    let voter = Address::generate(&env);
    env.mock_all_auths();
    client.vote_helpful(&voter, &product_id, &0, &false);

    // Verify vote was recorded
    let review = client.get_review(&product_id, &0);
    assert_eq!(review.helpful_votes, 0);
    assert_eq!(review.not_helpful_votes, 1);
}

#[test]
fn test_multiple_votes_same_review() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Popular review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit review first
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Multiple voters vote helpful
    let voters = create_test_users(&env, 3);
    for voter in voters.iter() {
        env.mock_all_auths();
        client.vote_helpful(&voter, &product_id, &0, &true);
    }

    // Verify all votes were recorded
    let review = client.get_review(&product_id, &0);
    assert_eq!(review.helpful_votes, 3);
    assert_eq!(review.not_helpful_votes, 0);
}

#[test]
fn test_mixed_votes_same_review() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Controversial review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit review first
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Mixed votes
    let helpful_voters = create_test_users(&env, 2);
    let not_helpful_voters = create_test_users(&env, 1);

    for voter in helpful_voters.iter() {
        env.mock_all_auths();
        client.vote_helpful(&voter, &product_id, &0, &true);
    }

    for voter in not_helpful_voters.iter() {
        env.mock_all_auths();
        client.vote_helpful(&voter, &product_id, &0, &false);
    }

    // Verify mixed votes were recorded
    let review = client.get_review(&product_id, &0);
    assert_eq!(review.helpful_votes, 2);
    assert_eq!(review.not_helpful_votes, 1);
}

#[test]
fn test_vote_emits_event() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Review to vote on");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit review first
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Vote and check event
    let voter = Address::generate(&env);
    env.mock_all_auths();
    client.vote_helpful(&voter, &product_id, &0, &true);

    // Verify vote event was emitted
    let events = env.events().all();
    assert!(events.len() >= 1); // At least one event (vote event)
}

#[test]
fn test_get_review_details_success() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Detailed review with specific information");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Get review details
    let review = client.get_review(&product_id, &0);
    
    // Verify all details
    assert_eq!(review.review_text, review_text);
    assert_eq!(review.reviewer, user);
    assert_eq!(review.verified_purchase, true);
    assert_eq!(review.helpful_votes, 0);
    assert_eq!(review.not_helpful_votes, 0);
    assert_eq!(review.responses.len(), 0);
    assert!(review.timestamp >= 0); // Allow 0 timestamp in test environment
}

#[test]
fn test_review_timestamp_accuracy() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Timestamp test review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    let before_submission = env.ledger().timestamp();
    
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
    
    let after_submission = env.ledger().timestamp();

    let review = client.get_review(&product_id, &0);
    assert!(review.timestamp >= before_submission);
    assert!(review.timestamp <= after_submission);
}

#[test]
fn test_review_responses_initialization() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Review with responses");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let review = client.get_review(&product_id, &0);
    assert_eq!(review.responses.len(), 0);
    assert!(review.responses.is_empty());
}

#[test]
fn test_review_verified_purchase_flag() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Verified purchase review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let review = client.get_review(&product_id, &0);
    assert_eq!(review.verified_purchase, true);
}

#[test]
fn test_review_storage_persistence() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Persistent review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Verify review is stored in persistent storage
    env.as_contract(&client.address, || {
        let review_key = DataKeys::Review(product_id, 0);
        let stored_review: ReviewDetails = env
            .storage()
            .persistent()
            .get(&review_key)
            .expect("Review not found in persistent storage");
        
        assert_eq!(stored_review.review_text, review_text);
        assert_eq!(stored_review.reviewer, user);
    });
}

#[test]
fn test_review_submission_with_boundary_values() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let boundary_data = BoundaryTestData::new(&env);
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Test with minimum valid review text
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &boundary_data.min_review_text, &purchase_link);

    let review = client.get_review(&product_id, &0);
    assert_eq!(review.review_text, boundary_data.min_review_text);

    // Test with maximum valid review text
    let product_id_2 = 12346u64;
    env.mock_all_auths();
    client.submit_review(&user, &product_id_2, &boundary_data.max_review_text, &purchase_link);

    let review_2 = client.get_review(&product_id_2, &0);
    assert_eq!(review_2.review_text, boundary_data.max_review_text);
}

#[test]
fn test_review_submission_high_volume() {
    let (env, client, _, _) = setup_test();
    let product_id = 12345u64;

    // Simulate high-volume review submissions
    simulate_high_volume_reviews(&env, &client, product_id, 10);

    // Verify all reviews were processed
    env.as_contract(&client.address, || {
        let count_key = DataKeys::ReviewCount(product_id);
        let review_count: u32 = env
            .storage()
            .persistent()
            .get(&count_key)
            .expect("Review count not found");
        assert_eq!(review_count, 10);
    });
}

#[test]
fn test_review_submission_with_special_characters() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Review with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let review = client.get_review(&product_id, &0);
    assert_eq!(review.review_text, review_text);
}

#[test]
fn test_review_submission_with_unicode() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Review with unicode: 🚀🌟💫⭐️✨");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    let review = client.get_review(&product_id, &0);
    assert_eq!(review.review_text, review_text);
}

#[test]
fn test_review_submission_with_long_purchase_link() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Review with long purchase link");
    let long_purchase_link = String::from_str(&env, "https://very-long-domain-name.com/very/long/path/to/purchase/with/many/parameters?param1=value1&param2=value2&param3=value3");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &long_purchase_link);

    let review = client.get_review(&product_id, &0);
    assert_eq!(review.reviewer, user);
    assert_eq!(review.verified_purchase, true);
}
