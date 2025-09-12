use crate::rating::calculate_weighted_rating;

use crate::tests::utils::TestCtx;

#[test]
#[should_panic(expected = "No rating available")]
fn no_ratings_panics_on_aggregation() {
    let ctx = TestCtx::new();
    let seller = ctx.gen_addr();
    ctx.env.as_contract(&ctx.contract_id, || {
        // directly calling lower-level function to match panic
        calculate_weighted_rating(ctx.env.clone(), seller.clone());
    });
}

#[test]
fn weighted_average_computation() {
    let ctx = TestCtx::new();
    let client = ctx.client();
    let seller = ctx.gen_addr();
    let b1 = ctx.gen_addr();
    let b2 = ctx.gen_addr();

    // ratings: (5*2 + 3*1) / (2+1) = 13/3 ≈ 4.33 -> scaled 433
    client.rate_seller(&seller, &b1, &5, &2, &None);
    client.rate_seller(&seller, &b2, &3, &1, &None);

    let scaled = ctx.env.as_contract(&ctx.contract_id, || {
        calculate_weighted_rating(ctx.env.clone(), seller.clone())
    });
    assert_eq!(scaled, 433);
}
