use crate::tests::utils::TestCtx;

#[test]
#[should_panic(expected = "Buyer and seller cannot be the same address")]
fn cannot_rate_self() {
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    client.rate_seller(&seller, &seller, &3, &1, &None);
}

#[test]
#[should_panic(expected = "Rating must be between 1 and 5")]
fn invalid_rating_out_of_range_low() {
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    let buyer = ctx.gen_addr();
    client.rate_seller(&seller, &buyer, &0, &1, &None);
}

#[test]
#[should_panic(expected = "Rating must be between 1 and 5")]
fn invalid_rating_out_of_range_high() {
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    let buyer = ctx.gen_addr();
    client.rate_seller(&seller, &buyer, &6, &1, &None);
}

#[test]
fn rating_submission_persists_and_accumulates() {
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    let buyer = ctx.gen_addr();

    // submit 3 ratings
    client.rate_seller(&seller, &buyer, &5, &2, &None);
    client.rate_seller(&seller, &buyer, &3, &1, &None);
    client.rate_seller(&seller, &buyer, &4, &0, &None); // zero weight still recorded

    let (rating_key, weighted_key, _) = ctx.seller_keys(&seller);

    ctx.env.as_contract(&ctx.contract_id, || {
        let ratings: soroban_sdk::Vec<crate::rating::Rating> = ctx
            .env
            .storage()
            .instance()
            .get(&rating_key)
            .expect("ratings stored");
        assert_eq!(ratings.len(), 3);

        let (tw_rating, tw): (u32, u32) = ctx
            .env
            .storage()
            .instance()
            .get(&weighted_key)
            .expect("weighted stored");
        assert_eq!(tw_rating, 5 * 2 + 3 * 1);
        assert_eq!(tw, 2 + 1);
    });
}

#[test]
fn duplicate_rating_is_allowed_currently() {
    // current contract allows same buyer to submit multiple ratings; document behavior
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    let buyer = ctx.gen_addr();

    client.rate_seller(&seller, &buyer, &5, &1, &None);
    client.rate_seller(&seller, &buyer, &4, &1, &None);

    let (rating_key, _, _) = ctx.seller_keys(&seller);
    ctx.env.as_contract(&ctx.contract_id, || {
        let ratings: soroban_sdk::Vec<crate::rating::Rating> = ctx
            .env
            .storage()
            .instance()
            .get(&rating_key)
            .unwrap();
        assert_eq!(ratings.len(), 2);
    });
}
