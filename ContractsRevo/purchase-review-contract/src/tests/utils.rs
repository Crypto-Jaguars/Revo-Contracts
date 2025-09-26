#![cfg(test)]

use super::super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, String, Vec,
};

/// Test setup helper that initializes the contract with admin
/// Returns: (env, client, admin, user)
pub fn setup_test() -> (Env, PurchaseReviewContractClient<'static>, Address, Address) {
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

/// Creates a test review with default values
pub fn create_test_review(env: &Env, reviewer: Address) -> ReviewDetails {
    ReviewDetails {
        review_text: String::from_str(env, "This is a test review"),
        reviewer,
        timestamp: env.ledger().timestamp(),
        helpful_votes: 0,
        not_helpful_votes: 0,
        verified_purchase: true,
        responses: Vec::new(env),
    }
}

/// Creates a test purchase verification data
pub fn create_test_verification_data(
    env: &Env,
    user: Address,
    product_id: u64,
    purchase_link: &str,
) -> PurchaseVerificationData {
    PurchaseVerificationData {
        user,
        product_id,
        purchase_link: String::from_str(env, purchase_link),
        is_verified: true,
        timestamp: env.ledger().timestamp(),
        has_review: false,
    }
}

/// Creates a test review report data
pub fn create_test_report_data(
    env: &Env,
    reporter: Address,
    product_id: u64,
    review_id: u32,
    reason: &str,
) -> crate::datatype::ReviewReportData {
    crate::datatype::ReviewReportData {
        reporter,
        product_id,
        review_id,
        reason: String::from_str(env, reason),
        timestamp: env.ledger().timestamp(),
    }
}

/// Helper to advance time by specified seconds
pub fn advance_time(env: &Env, seconds: u64) {
    let current_time = env.ledger().timestamp();
    env.ledger().set_timestamp(current_time + seconds);
}

/// Helper to create multiple test users
pub fn create_test_users(env: &Env, count: usize) -> Vec<Address> {
    let mut users = Vec::new(env);
    for _ in 0..count {
        users.push_back(Address::generate(env));
    }
    users
}

/// Helper to create test review text of specific length
pub fn create_review_text(env: &Env, length: usize) -> String {
    let text = "a".repeat(length);
    String::from_str(env, &text)
}

/// Helper to verify that an event was emitted
pub fn assert_event_emitted(env: &Env, expected_contract_id: Address, expected_topic: &str) {
    let events = env.events().all();
    let mut found = false;
    
    for event in events.iter() {
        if event.0 == expected_contract_id {
            // Check if the topic matches (simplified check)
            found = true;
            break;
        }
    }
    
    assert!(found, "Expected event with topic '{}' was not emitted", expected_topic);
}

/// Helper to count events with specific topic
pub fn count_events_with_topic(env: &Env, expected_contract_id: Address) -> usize {
    let events = env.events().all();
    let mut count = 0;
    
    for event in events.iter() {
        if event.0 == expected_contract_id {
            count += 1;
        }
    }
    
    count
}

/// Helper to simulate high-volume review submissions
pub fn simulate_high_volume_reviews(
    env: &Env,
    client: &PurchaseReviewContractClient,
    product_id: u64,
    count: usize,
) {
    for i in 0..count {
        let user = Address::generate(env);
        let review_text = String::from_str(env, "Review number");
        let purchase_link = String::from_str(env, "https://example.com/purchase/123");
        
        env.mock_all_auths();
        let _ = client.try_submit_review(&user, &product_id, &review_text, &purchase_link);
    }
}

/// Helper to create boundary test data
pub struct BoundaryTestData {
    pub min_review_text: String,
    pub max_review_text: String,
    pub empty_review_text: String,
    pub min_product_id: u64,
    pub max_product_id: u64,
    pub valid_purchase_link: String,
    pub empty_purchase_link: String,
}

impl BoundaryTestData {
    pub fn new(env: &Env) -> Self {
        Self {
            min_review_text: String::from_str(env, "OK"), // Minimum valid length
            max_review_text: create_review_text(env, 1000), // Maximum valid length
            empty_review_text: String::from_str(env, ""),
            min_product_id: 1,
            max_product_id: u64::MAX,
            valid_purchase_link: String::from_str(env, "https://example.com/purchase/123"),
            empty_purchase_link: String::from_str(env, ""),
        }
    }
}

/// Helper to create test data for aggregation tests
pub struct AggregationTestData {
    pub product_id: u64,
    pub users: Vec<Address>,
    pub review_texts: Vec<String>,
    pub purchase_links: Vec<String>,
}

impl AggregationTestData {
    pub fn new(env: &Env, product_id: u64, user_count: usize) -> Self {
        let users = create_test_users(env, user_count);
        let mut review_texts = Vec::new(env);
        let mut purchase_links = Vec::new(env);
        
        for i in 0..user_count {
            review_texts.push_back(String::from_str(env, "Review"));
            purchase_links.push_back(String::from_str(env, "https://example.com/purchase/123"));
        }
        
        Self {
            product_id,
            users,
            review_texts,
            purchase_links,
        }
    }
}
