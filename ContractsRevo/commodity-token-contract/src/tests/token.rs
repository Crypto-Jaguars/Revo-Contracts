#![cfg(test)]
use soroban_sdk::{BytesN, String};

use crate::{issue::IssueError, storage, CommodityTokenContract};

use crate::tests::utils::TestContext;

#[test]
fn token_creation_and_metadata() {
    let ctx = TestContext::new();
    ctx.init_with_admin();

    // Provision inventory and verification
    ctx.add_inventory("COFFEE", 1_000);
    let verification = ctx.register_verification("COFFEE", [9u8; 32]);

    // Authorized issuer: admin by default
    let issuer = ctx.admin.clone();

    // Issue a token
    let token_id = ctx.issue_token(&issuer, "COFFEE", 250, "AA", "WH-1", 60 * 60, &verification);

    // Validate metadata and storage
    let token = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::get_token_metadata(ctx.env.clone(), token_id.clone()).unwrap()
    });
    assert_eq!(token.commodity_type, String::from_str(&ctx.env, "COFFEE"));
    assert_eq!(token.quantity, 250);
    assert_eq!(token.grade, String::from_str(&ctx.env, "AA"));

    // Token listed under commodity index
    let listed = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::list_tokens_by_commodity(
            ctx.env.clone(),
            String::from_str(&ctx.env, "COFFEE"),
        )
    });
    assert!(listed.iter().any(|id| id == token_id));

    // Owner is issuer
    let owner = ctx.env.as_contract(&ctx.contract_id, || {
        storage::get_token_owner(&ctx.env, &token_id).unwrap()
    });
    assert_eq!(owner, issuer);

    // Details contain validity
    let details = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::get_token_details(ctx.env.clone(), token_id.clone()).unwrap()
    });
    let valid: bool = details
        .get(String::from_str(&ctx.env, "valid"))
        .unwrap()
        .try_into()
        .unwrap();
    assert!(valid);
}

#[test]
fn duplicate_product_id_not_possible_unique_ids() {
    let ctx = TestContext::new();
    ctx.init_with_admin();
    ctx.add_inventory("WHEAT", 5_000);
    let verification = ctx.register_verification("WHEAT", [7u8; 32]);
    let issuer = ctx.admin.clone();

    // Issue two tokens with same inputs; nonce ensures unique ids
    let t1 = ctx.issue_token(&issuer, "WHEAT", 100, "A", "WH", 3600, &verification);
    let t2 = ctx.issue_token(&issuer, "WHEAT", 100, "A", "WH", 3600, &verification);
    assert_ne!(t1, t2, "token ids must be unique even for same metadata");
}

#[test]
fn invalid_metadata_or_expiration_rejected() {
    let ctx = TestContext::new();
    ctx.init_with_admin();
    ctx.add_inventory("CORN", 1000);

    // No verification registered -> InvalidCommodityData
    let issuer = ctx.admin.clone();
    ctx.set_time(1_000_000);
    let now = ctx.now();
    let res = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::issue_token(
            ctx.env.clone(),
            issuer.clone(),
            String::from_str(&ctx.env, "CORN"),
            10,
            String::from_str(&ctx.env, "A"),
            String::from_str(&ctx.env, "WH"),
            now + 3600,
            BytesN::from_array(&ctx.env, &[0u8; 32]),
        )
    });
    assert_eq!(res.unwrap_err(), IssueError::InvalidCommodityData);

    // Register verification, but set expiration in the past -> InvalidExpirationDate
    let verification = ctx.register_verification("CORN", [1u8; 32]);
    let past_res = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::issue_token(
            ctx.env.clone(),
            issuer.clone(),
            String::from_str(&ctx.env, "CORN"),
            10,
            String::from_str(&ctx.env, "A"),
            String::from_str(&ctx.env, "WH"),
            now - 1,
            verification.clone(),
        )
    });
    assert_eq!(past_res.unwrap_err(), IssueError::InvalidExpirationDate);
}
