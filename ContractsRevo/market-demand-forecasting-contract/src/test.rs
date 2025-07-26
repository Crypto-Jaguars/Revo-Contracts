#![cfg(test)]

use super::*;
use crate::error::ContractError;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Ledger as _}, vec, Address, BytesN, Env, IntoVal, String
};

// --- Test Struct and Setup ---

struct ForecastingTest<'a> {
    env: Env,
    admin: Address,
    oracle: Address,
    contract: MarketDemandForecastingContractClient<'a>,
}

impl<'a> ForecastingTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        let contract_id = env.register_contract(None, MarketDemandForecastingContract);
        let contract = MarketDemandForecastingContractClient::new(&env, &contract_id);

        contract.initialize(&admin);
        contract.set_oracle(&admin, &oracle);

        ForecastingTest {
            env,
            admin,
            oracle,
            contract,
        }
    }
}

fn current_time() -> u64 {
    1721763480
}

// --- Tests ---

#[test]
fn test_initialize_and_set_oracle() {
    let test = ForecastingTest::setup();

    // Try to initialize again
    let init_result = test.contract.try_initialize(&test.admin);
    assert_eq!(init_result, Err(Ok(ContractError::AlreadyInitialized)));

    // Try to set oracle as non-admin
    let non_admin = Address::generate(&test.env);
    let oracle_result = test.contract.try_set_oracle(&non_admin, &test.oracle);
    assert_eq!(oracle_result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_register_product() {
    let test = ForecastingTest::setup();

    let name = String::from_str(&test.env, "Wheat");
    let historical_demand = vec![&test.env, 1000, 1200, 1100];
    let product_id = test
        .contract
        .register_product(&name, &historical_demand);

    let product = test.contract.get_product(&product_id);
    assert_eq!(product.name, name);
    assert_eq!(product.historical_demand.len(), 3);
}

#[test]
fn test_register_product_invalid_data() {
    let test = ForecastingTest::setup();

    let name = String::from_str(&test.env, ""); // Empty name
    let historical_demand = vec![&test.env, 1000, 1200, 1100];
    let result = test.contract.try_register_product(&name, &historical_demand);
    assert_eq!(result, Err(Ok(ContractError::InvalidData)));
}

#[test]
fn test_generate_forecast() {
    let test = ForecastingTest::setup();

    // Register a product first
    let product_id = test.contract.register_product(
        &String::from_str(&test.env, "Corn"),
        &vec![&test.env, 2000, 2100, 1900],
    );

    let region = String::from_str(&test.env, "North");
    let data_hash = BytesN::random(&test.env);
    let predicted_demand_from_oracle = 2500_i128; // This value comes from the oracle

    let forecast_id = test.contract.generate_forecast(
        &test.oracle,
        &product_id,
        &region,
        &predicted_demand_from_oracle,
        &data_hash,
    );

    let forecast = test.contract.get_forecast(&forecast_id);
    assert_eq!(forecast.product_id, product_id);
    assert_eq!(forecast.region, region);
    // Assert that the stored demand is the one provided by the oracle, not a calculation.
    assert_eq!(forecast.predicted_demand, predicted_demand_from_oracle);
}

#[test]
fn test_generate_forecast_unauthorized_oracle() {
    let test = ForecastingTest::setup();
    let product_id = test.contract.register_product(
        &String::from_str(&test.env, "Corn"),
        &vec![&test.env, 2000, 2100, 1900],
    );
    let unauthorized_oracle = Address::generate(&test.env);

    let result = test.contract.try_generate_forecast(
        &unauthorized_oracle,
        &product_id,
        &String::from_str(&test.env, "North"),
        &1234, // predicted_demand
        &BytesN::random(&test.env),
    );
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_generate_forecast_product_not_found() {
    let test = ForecastingTest::setup();
    let fake_product_id = BytesN::random(&test.env);

    let result = test.contract.try_generate_forecast(
        &test.oracle,
        &fake_product_id,
        &String::from_str(&test.env, "North"),
        &1234, // predicted_demand
        &BytesN::random(&test.env),
    );
    assert_eq!(result, Err(Ok(ContractError::ProductNotFound)));
}

#[test]
fn test_list_forecasts() {
    let test = ForecastingTest::setup();

    // Register products
    let p1_id = test.contract.register_product(&"P1".into_val(&test.env), &vec![&test.env, 100]);
    let p2_id = test.contract.register_product(&"P2".into_val(&test.env), &vec![&test.env, 200]);

    // Generate forecasts
    test.contract.generate_forecast(&test.oracle, &p1_id, &"North".into_val(&test.env), &150, &BytesN::random(&test.env));
    test.contract.generate_forecast(&test.oracle, &p2_id, &"North".into_val(&test.env), &250, &BytesN::random(&test.env));
    test.contract.generate_forecast(&test.oracle, &p1_id, &"South".into_val(&test.env), &180, &BytesN::random(&test.env));

    // Test listing all
    assert_eq!(test.contract.list_forecasts(&None, &None).len(), 3);

    // Test filtering by region
    let north_forecasts = test.contract.list_forecasts(&None, &Some("North".into_val(&test.env)));
    assert_eq!(north_forecasts.len(), 2);

    // Test filtering by product
    let p1_forecasts = test.contract.list_forecasts(&Some(p1_id.clone()), &None);
    assert_eq!(p1_forecasts.len(), 2);

    // Test filtering by both
    let p1_north_forecasts = test.contract.list_forecasts(&Some(p1_id), &Some("North".into_val(&test.env)));
    assert_eq!(p1_north_forecasts.len(), 1);
}

#[test]
fn test_generate_recommendation_with_recency_and_averaging() {
    let test = ForecastingTest::setup();

    // --- Setup ---
    let region = String::from_str(&test.env, "Midwest");
    let seven_days = 7;

    let corn_id = test.contract.register_product(&"Corn".into_val(&test.env), &vec![&test.env, 0]);
    let wheat_id = test.contract.register_product(&"Wheat".into_val(&test.env), &vec![&test.env, 0]);

    // --- Simulate Time and Forecasts ---

    // 1. An old forecast for Wheat (should be ignored)
    test.env.ledger().with_mut(|li| {
        li.timestamp = 10_000;
    });
    test.contract.generate_forecast(&test.oracle, &wheat_id, &region, &9999, &BytesN::random(&test.env));

    // 2. Recent forecasts for Corn (average should be 1100)
    test.env.ledger().with_mut(|li| {
        li.timestamp = current_time() - (60 * 60 * 24 * 2); // 2 days ago
    });
    test.contract.generate_forecast(&test.oracle, &corn_id, &region, &1000, &BytesN::random(&test.env));
    test.env.ledger().with_mut(|li| {
        li.timestamp = current_time() - (60 * 60 * 24 * 1); // 1 day ago
    });
    test.contract.generate_forecast(&test.oracle, &corn_id, &region, &1200, &BytesN::random(&test.env));

    // 3. Recent forecast for Wheat (average should be 2000)
    test.env.ledger().with_mut(|li| {
        li.timestamp = current_time() - (60 * 60 * 24 * 3); // 3 days ago
    });
    test.contract.generate_forecast(&test.oracle, &wheat_id, &region, &2000, &BytesN::random(&test.env));

    // Set ledger time back to current for the recommendation call
    test.env.ledger().with_mut(|li| {
        li.timestamp = current_time();
    });

    // --- Generate and Verify Recommendation ---
    let recommendations = test.contract.generate_recommendation(&region, &seven_days);

    // Should only be two products with recent forecasts
    assert_eq!(recommendations.len(), 2);

    // Wheat (avg 2000) should be ranked higher than Corn (avg 1100)
    assert_eq!(recommendations.get(0).unwrap().product_id, wheat_id);
    assert_eq!(recommendations.get(1).unwrap().product_id, corn_id);
}


#[test]
fn test_generate_recommendation_region_not_found() {
    let test = ForecastingTest::setup();
    let region = String::from_str(&test.env, "Atlantis");
    let result = test.contract.try_generate_recommendation(&region, &7);
    assert_eq!(result, Err(Ok(ContractError::RegionNotFound)));
}
