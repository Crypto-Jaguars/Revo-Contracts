#![cfg(test)]

use super::super::*;
use super::utils::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, String,
};

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_submit_review_empty_text() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_submit_review_text_too_long() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = create_review_text(&env, 1001); // Exceeds 1000 character limit
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_submit_review_empty_purchase_link() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Valid review text");
    let purchase_link = String::from_str(&env, "");

    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_submit_review_duplicate_purchase_verification() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "First review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // First submission should succeed
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Second submission with same purchase link should fail
    let review_text_2 = String::from_str(&env, "Second review");
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text_2, &purchase_link);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_submit_review_already_reviewed() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "First review");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit first review
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Try to submit another review for the same purchase with different link
    // This should fail because the purchase is already verified
    let review_text_2 = String::from_str(&env, "Second review");
    let purchase_link_2 = String::from_str(&env, "https://example.com/purchase/12346");
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text_2, &purchase_link_2);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_vote_helpful_review_not_found() {
    let (env, client, _, _) = setup_test();
    let product_id = 12345u64;
    let review_id = 999u32; // Non-existent review
    let voter = Address::generate(&env);

    env.mock_all_auths();
    client.vote_helpful(&voter, &product_id, &review_id, &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_vote_helpful_already_voted() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let review_text = String::from_str(&env, "Review to vote on");
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Submit review first
    env.mock_all_auths();
    client.submit_review(&user, &product_id, &review_text, &purchase_link);

    // Vote helpful
    let voter = Address::generate(&env);
    env.mock_all_auths();
    client.vote_helpful(&voter, &product_id, &0, &true);

    // Try to vote again - should fail
    env.mock_all_auths();
    client.vote_helpful(&voter, &product_id, &0, &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_get_review_not_found() {
    let (env, client, _, _) = setup_test();
    let product_id = 12345u64;
    let review_id = 999u32; // Non-existent review

    client.get_review(&product_id, &review_id);
}

#[test]
fn test_purchase_verification_success() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    let result = client.verify_purchase(&user, &product_id, &purchase_link);
    assert!(result);

    // Verify purchase is marked as verified
    assert!(client.is_purchase_verified(&user, &product_id));
}

#[test]
fn test_purchase_verification_emits_event() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &purchase_link);

    // Verify event was emitted
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let event = events.get(0).unwrap();
    assert_eq!(event.0, client.address);
}

#[test]
fn test_purchase_verification_storage() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &purchase_link);

    // Verify verification data is stored
    env.as_contract(&client.address, || {
        let key = DataKeys::PurchaseVerification(product_id, user.clone());
        let verification_data: PurchaseVerificationData = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Verification data not found");
        
        assert_eq!(verification_data.user, user);
        assert_eq!(verification_data.product_id, product_id);
        assert_eq!(verification_data.purchase_link, purchase_link);
        assert_eq!(verification_data.is_verified, true);
        assert_eq!(verification_data.has_review, false);
    });
}

#[test]
fn test_purchase_verification_timestamp() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    let before_verification = env.ledger().timestamp();
    
    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &purchase_link);
    
    let after_verification = env.ledger().timestamp();

    // Verify timestamp is within expected range
    env.as_contract(&client.address, || {
        let key = DataKeys::PurchaseVerification(product_id, user.clone());
        let verification_data: PurchaseVerificationData = env
            .storage()
            .persistent()
            .get(&key)
            .expect("Verification data not found");
        
        assert!(verification_data.timestamp >= before_verification);
        assert!(verification_data.timestamp <= after_verification);
    });
}

#[test]
fn test_purchase_verification_multiple_products() {
    let (env, client, _, user) = setup_test();
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");

    // Verify purchases for different products
    for product_id in 1..=3 {
        env.mock_all_auths();
        client.verify_purchase(&user, &product_id, &purchase_link);
        
        assert!(client.is_purchase_verified(&user, &product_id));
    }
}

#[test]
fn test_purchase_verification_multiple_users() {
    let (env, client, _, _) = setup_test();
    let product_id = 12345u64;
    let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");
    let users = create_test_users(&env, 3);

    // Verify purchases for different users
    for user in users.iter() {
        env.mock_all_auths();
        client.verify_purchase(&user, &product_id, &purchase_link);
        
        assert!(client.is_purchase_verified(&user, &product_id));
    }
}

#[test]
fn test_purchase_verification_different_links() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;

    // Verify purchases with different links
    for i in 1..=3 {
        let purchase_link = String::from_str(&env, "https://example.com/purchase/123");
        
        env.mock_all_auths();
        client.verify_purchase(&user, &product_id, &purchase_link);
        
        assert!(client.is_purchase_verified(&user, &product_id));
    }
}

#[test]
fn test_purchase_verification_boundary_values() {
    let (env, client, _, user) = setup_test();
    let boundary_data = BoundaryTestData::new(&env);

    // Test with minimum product ID
    env.mock_all_auths();
    client.verify_purchase(&user, &boundary_data.min_product_id, &boundary_data.valid_purchase_link);
    assert!(client.is_purchase_verified(&user, &boundary_data.min_product_id));

    // Test with maximum product ID
    env.mock_all_auths();
    client.verify_purchase(&user, &boundary_data.max_product_id, &boundary_data.valid_purchase_link);
    assert!(client.is_purchase_verified(&user, &boundary_data.max_product_id));
}

#[test]
fn test_purchase_verification_long_link() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let long_purchase_link = String::from_str(&env, "https://very-long-domain-name.com/very/long/path/to/purchase/with/many/parameters?param1=value1&param2=value2&param3=value3");

    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &long_purchase_link);
    
    assert!(client.is_purchase_verified(&user, &product_id));
}

#[test]
fn test_purchase_verification_special_characters() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let special_char_link = String::from_str(&env, "https://example.com/purchase/12345?special=!@#$%^&*()_+-=[]{}|;':\",./<>?");

    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &special_char_link);
    
    assert!(client.is_purchase_verified(&user, &product_id));
}

#[test]
fn test_purchase_verification_unicode() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let unicode_link = String::from_str(&env, "https://example.com/purchase/12345?unicode=🚀🌟💫⭐️✨");

    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &unicode_link);
    
    assert!(client.is_purchase_verified(&user, &product_id));
}

#[test]
fn test_purchase_verification_https_http() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;

    // Test HTTPS link
    let https_link = String::from_str(&env, "https://example.com/purchase/12345");
    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &https_link);
    assert!(client.is_purchase_verified(&user, &product_id));

    // Test HTTP link
    let product_id_2 = 12346u64;
    let http_link = String::from_str(&env, "http://example.com/purchase/12345");
    env.mock_all_auths();
    client.verify_purchase(&user, &product_id_2, &http_link);
    assert!(client.is_purchase_verified(&user, &product_id_2));
}

#[test]
fn test_purchase_verification_different_domains() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;

    let domains = [
        "https://amazon.com/purchase/12345",
        "https://ebay.com/purchase/12345",
        "https://shopify.com/purchase/12345",
        "https://custom-store.com/purchase/12345",
    ];

    for (i, domain) in domains.iter().enumerate() {
        let purchase_link = String::from_str(&env, domain);
        env.mock_all_auths();
        client.verify_purchase(&user, &(product_id + i as u64), &purchase_link);
        assert!(client.is_purchase_verified(&user, &(product_id + i as u64)));
    }
}

#[test]
fn test_purchase_verification_with_fragments() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let fragment_link = String::from_str(&env, "https://example.com/purchase/12345#section1");

    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &fragment_link);
    
    assert!(client.is_purchase_verified(&user, &product_id));
}

#[test]
fn test_purchase_verification_with_ports() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;
    let port_link = String::from_str(&env, "https://example.com:8080/purchase/12345");

    env.mock_all_auths();
    client.verify_purchase(&user, &product_id, &port_link);
    
    assert!(client.is_purchase_verified(&user, &product_id));
}

#[test]
fn test_purchase_verification_case_sensitivity() {
    let (env, client, _, user) = setup_test();
    let product_id = 12345u64;

    // Test different case variations
    let links = [
        "https://EXAMPLE.COM/purchase/12345",
        "https://example.com/PURCHASE/12345",
        "https://Example.Com/Purchase/12345",
    ];

    for (i, link) in links.iter().enumerate() {
        let purchase_link = String::from_str(&env, link);
        env.mock_all_auths();
        client.verify_purchase(&user, &(product_id + i as u64), &purchase_link);
        assert!(client.is_purchase_verified(&user, &(product_id + i as u64)));
    }
}

#[test]
fn test_purchase_verification_high_volume() {
    let (env, client, _, user) = setup_test();
    let base_product_id = 12345u64;

    // Verify many purchases
    for i in 0..100 {
        let product_id = base_product_id + i;
        let purchase_link = String::from_str(&env, "https://example.com/purchase/123");
        
        env.mock_all_auths();
        client.verify_purchase(&user, &product_id, &purchase_link);
        assert!(client.is_purchase_verified(&user, &product_id));
    }
}

#[test]
fn test_purchase_verification_concurrent_users() {
    let (env, client, _, _) = setup_test();
    let product_id = 12345u64;
    let users = create_test_users(&env, 50);

    // Verify purchases for many users concurrently
    for user in users.iter() {
        let purchase_link = String::from_str(&env, "https://example.com/purchase/12345");
        env.mock_all_auths();
        client.verify_purchase(&user, &product_id, &purchase_link);
        assert!(client.is_purchase_verified(&user, &product_id));
    }
}
