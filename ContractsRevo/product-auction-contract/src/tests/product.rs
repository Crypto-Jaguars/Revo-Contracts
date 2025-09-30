use crate::datatype::Condition;
use crate::tests::utils::*;
use soroban_sdk::{testutils::Address as _, vec, Address, String, Symbol};

#[test]
fn test_add_product_with_different_conditions() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let conditions = vec![
        &env,
        Condition::New,
        Condition::OpenBox,
        Condition::UsedGood,
        Condition::UsedAcceptable,
        Condition::Refurbished,
    ];

    for condition in conditions.iter() {
        let name = Symbol::new(&env, "Product");
        let description = String::from_str(&env, "Test description text here");
        let price = 100u64;
        let stock = 5u32;
        let images = vec![&env, String::from_str(&env, "img.jpg")];
        let weight = 10u64;

        let product_id = client.add_product(
            &seller, &name, &description, &price, &condition, &stock, &images, &weight,
        );

        let product = client.get_product(&seller, &product_id);
        assert_eq!(product.condition, condition);
    }
}

#[test]
fn test_add_product_maximum_description_length() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let long_description = "a".repeat(500);
    let description = String::from_str(&env, &long_description);

    let name = Symbol::new(&env, "Product");
    let price = 100u64;
    let condition = Condition::New;
    let stock = 5u32;
    let images = vec![&env, String::from_str(&env, "img.jpg")];
    let weight = 10u64;

    let product_id = client.add_product(
        &seller, &name, &description, &price, &condition, &stock, &images, &weight,
    );

    let product = client.get_product(&seller, &product_id);
    assert_eq!(product.description.len(), 500);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_add_product_description_too_long() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let long_description = "a".repeat(501);
    let description = String::from_str(&env, &long_description);

    let name = Symbol::new(&env, "Product");
    let price = 100u64;
    let condition = Condition::New;
    let stock = 5u32;
    let images = vec![&env, String::from_str(&env, "img.jpg")];
    let weight = 10u64;

    client.add_product(
        &seller, &name, &description, &price, &condition, &stock, &images, &weight,
    );
}

#[test]
fn test_update_stock_to_zero() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_test_product(&env, &client, &seller);
    
    client.update_stock(&seller, &product_id, &0u32);

    let product = client.get_product(&seller, &product_id);
    assert_eq!(product.stock, 0);
}

#[test]
fn test_update_stock_large_value() {
    let env = setup_env();
    let client = setup_contract(&env);
    let seller = Address::generate(&env);
    env.mock_all_auths();

    let product_id = create_test_product(&env, &client, &seller);
    
    client.update_stock(&seller, &product_id, &999999u32);

    let product = client.get_product(&seller, &product_id);
    assert_eq!(product.stock, 999999);
}