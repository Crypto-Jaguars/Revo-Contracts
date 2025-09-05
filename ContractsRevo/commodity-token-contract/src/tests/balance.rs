#![cfg(test)]
use soroban_sdk::String;

use crate::CommodityTokenContract;

use crate::tests::utils::TestContext;

#[test]
fn balance_queries_and_non_existent_account() {
    let ctx = TestContext::new();
    ctx.init_with_admin();

    // No inventory yet for TEA
    let inv = ctx.get_inventory("TEA");
    assert_eq!(inv.total_quantity, 0);
    assert_eq!(inv.available_quantity, 0);
    assert_eq!(inv.issued_tokens, 0);

    // Non-existent account has no tokens; we can try fetching owner of random token id and expect error
    let random = soroban_sdk::BytesN::from_array(&ctx.env, &[8u8; 32]);
    let missing = ctx.env.as_contract(&ctx.contract_id, || {
        crate::storage::get_token_owner(&ctx.env, &random)
    });
    assert_eq!(missing.unwrap_err(), crate::ContractError::OwnerNotFound);
}

#[test]
fn transfer_exceeding_available_balance_blocked() {
    let ctx = TestContext::new();
    ctx.init_with_admin();
    ctx.add_inventory("BARLEY", 150);
    let verification = ctx.register_verification("BARLEY", [6u8; 32]);

    // Issue 120 units token
    let _ = ctx.issue_token(&ctx.admin, "BARLEY", 120, "A", "WH", 3600, &verification);

    // Attempt to issue another token exceeding available inventory (remaining 30 only)
    let res = ctx.env.as_contract(&ctx.contract_id, || {
        CommodityTokenContract::issue_token(
            ctx.env.clone(),
            ctx.admin.clone(),
            String::from_str(&ctx.env, "BARLEY"),
            40,
            String::from_str(&ctx.env, "A"),
            String::from_str(&ctx.env, "WH"),
            ctx.env.ledger().timestamp() + 3600,
            verification.clone(),
        )
    });

    assert_eq!(res.unwrap_err(), crate::issue::IssueError::InsufficientInventory);
}
