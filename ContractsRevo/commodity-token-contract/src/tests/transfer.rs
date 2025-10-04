#![cfg(test)]
use soroban_sdk::{testutils::Address as _, Address, String};

use crate::{issue::IssueError, storage, CommodityTokenContract};

use crate::tests::utils::TestContext;

#[test]
fn unauthorized_issuer_cannot_issue() {
    let ctx = TestContext::new();
    ctx.init_with_admin();
    ctx.add_inventory("RICE", 500);
    let verification = ctx.register_verification("RICE", [2u8; 32]);

    // Random address not admin nor authorized issuer
    let unauthorized = Address::generate(&ctx.env);

    let res = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::issue_token(
            ctx.env.clone(),
            unauthorized.clone(),
            String::from_str(&ctx.env, "RICE"),
            50,
            String::from_str(&ctx.env, "A"),
            String::from_str(&ctx.env, "LOC"),
            ctx.env.ledger().timestamp() + 3600,
            verification.clone(),
        )
    });

    assert_eq!(res.unwrap_err(), IssueError::UnauthorizedIssuer);
}

#[test]
fn transfer_redeem_requires_owner_and_balance() {
    let ctx = TestContext::new();
    ctx.init_with_admin();
    ctx.add_inventory("BEANS", 200);
    let verification = ctx.register_verification("BEANS", [3u8; 32]);

    // Admin issues token and becomes owner
    let token_id = ctx.issue_token(&ctx.admin, "BEANS", 100, "A", "WH", 3600, &verification);

    // Another user tries to redeem (transfer out) without ownership
    let other = Address::generate(&ctx.env);
    let not_owner = ctx.env.as_contract(&ctx.contract_id, || {
        crate::redeem::redeem_token(&ctx.env, &token_id, &other, 10)
    });
    assert_eq!(not_owner.unwrap_err(), crate::RedeemError::NotTokenOwner);

    // Owner tries to redeem more than owned
    let over = ctx.env.as_contract(&ctx.contract_id, || {
        crate::redeem::redeem_token(&ctx.env, &token_id, &ctx.admin, 101)
    });
    assert_eq!(over.unwrap_err(), crate::RedeemError::InsufficientQuantity);

    // Zero-amount redeem is a no-op in our logic? Current code allows 0, test 0 leaves state
    let inv_before = ctx.get_inventory("BEANS");
    let zero = ctx.env.as_contract(&ctx.contract_id, || {
        crate::redeem::redeem_token(&ctx.env, &token_id, &ctx.admin, 0)
    });
    assert!(zero.is_ok());
    let inv_after = ctx.get_inventory("BEANS");
    assert_eq!(
        inv_before, inv_after,
        "zero redemption must not change inventory"
    );
}

#[test]
fn redeem_updates_balances_and_inventory() {
    let ctx = TestContext::new();
    ctx.init_with_admin();
    ctx.add_inventory("SOY", 1000);
    let verification = ctx.register_verification("SOY", [4u8; 32]);

    let token_id = ctx.issue_token(&ctx.admin, "SOY", 300, "A", "WH", 3600, &verification);

    // Redeem 100
    ctx.env.as_contract(&ctx.contract_id, || {
        crate::redeem::redeem_token(&ctx.env, &token_id, &ctx.admin, 100).unwrap();
    });

    // Token quantity decreased to 200
    let token = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::get_token_metadata(ctx.env.clone(), token_id.clone()).unwrap()
    });
    assert_eq!(token.quantity, 200);

    // Inventory: issued_tokens and total_quantity decreased by 100
    let inv = ctx.get_inventory("SOY");
    assert_eq!(inv.issued_tokens, 300 - 100);
    assert_eq!(inv.total_quantity, 1000 - 100);
}

#[test]
fn redeem_after_expiration_fails() {
    let ctx = TestContext::new();
    ctx.init_with_admin();
    ctx.add_inventory("COTTON", 100);
    let verification = ctx.register_verification("COTTON", [5u8; 32]);

    // Issue token that expires immediately
    let token_id = ctx.issue_token(&ctx.admin, "COTTON", 50, "A", "WH", 1, &verification);

    // Fast-forward time by manipulating ledger timestamp is not available directly here,
    // so simulate by redeeming after waiting: we assume timestamp monotonic but not controllable.
    // Instead, call redeem with normal flow but ensure expiration check is active by setting expires_in_secs small.
    // If the environment timestamp hasn't advanced beyond expiration yet, we still assert logic by manual check.
    // We enforce failure by faking a token past expiration: directly store expired token.
    ctx.set_time(1_000_000);
    ctx.env.as_contract(&ctx.contract_id, || {
        let mut tok = storage::get_token(&ctx.env, &token_id).unwrap();
        tok.expiration_date = ctx.now() - 1;
        storage::store_token(&ctx.env, &token_id, &tok);
    });

    let res = ctx.env.as_contract(&ctx.contract_id, || {
        crate::redeem::redeem_token(&ctx.env, &token_id, &ctx.admin, 10)
    });
    assert_eq!(res.unwrap_err(), crate::RedeemError::TokenExpired);
}
