use crate::tests::utils::TestCtx;

#[test]
#[should_panic(expected = "No rating available")]
fn reputation_on_non_rated_seller_panics() {
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    client.seller_reputation_score(&seller);
}

#[test]
fn reputation_follows_weighted_rating_and_history_updates() {
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    let b1 = ctx.gen_addr();

    // Single rating 5 with weight 2 -> weighted scaled 500 -> reputation 5
    client.rate_seller(&seller, &b1, &5, &2, &None);
    let rep1 = client.seller_reputation_score(&seller);
    assert_eq!(rep1, 5);

    // Add low rating 2 with weight 2 -> weighted: (10+4)/4=3.5 -> scaled 350 -> reputation 4
    client.rate_seller(&seller, &b1, &2, &2, &None);
    let rep2 = client.seller_reputation_score(&seller);
    assert_eq!(rep2, 4);

    // History updated twice
    let (_, _, hist_key) = ctx.seller_keys(&seller);
    ctx.env.as_contract(&ctx.contract_id, || {
        let history: soroban_sdk::Vec<crate::reputation::ReputationRecord> = ctx
            .env
            .storage()
            .instance()
            .get(&hist_key)
            .expect("history exists");
        assert_eq!(history.len(), 2);
    });
}
