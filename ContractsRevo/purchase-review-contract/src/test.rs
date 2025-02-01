#![cfg(test)]

use crate::{
    PurchaseReviewContract, PurchaseReviewContractClient,
};

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Bytes, BytesN, IntoVal, symbol_short, TryFromVal, Val, Vec
};

#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_panic_reviewer_not_authenticated() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = <Address>::generate(&env);
    let product_id = 12345;
    let review_text = String::from_str(&env, &"a".repeat(1001));
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic]
fn test_invalid_review_text_length_empty() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = <Address>::generate(&env);
    let product_id = 12345;
    let review_text = String::from_str(&env, "");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic]
fn test_invalid_review_text_length_too_long() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = <Address>::generate(&env);
    let product_id = 12345;
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

    let user = <Address>::generate(&env);
    let product_id = 12345u128;
    let review_text = String::from_str(&env, "This product is excellent!");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

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
        assert_eq!(
            stored_review.timestamp,
            env.ledger().timestamp()
        );
    });
}

#[test]
#[should_panic]
fn test_duplicate_review_submission() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = <Address>::generate(&env);
    let product_id = 12345;
    let review_text = String::from_str(&env, "Great product!");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic]
fn test_invalid_purchase_link() {
    let env = Env::default();
    let contract_id = env.register(PurchaseReviewContract, ());
    let client = PurchaseReviewContractClient::new(&env, &contract_id);

    let user = <Address>::generate(&env);
    let product_id = 12345;
    let review_text = String::from_str(&env, "Great product!");
    let purchase_link = String::from_str(&env, "");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}
