#![cfg(test)]
use crate::datatype::Condition;

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

fn setup_test(mock_auths: bool) -> (Env, ProductAuctionContractClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(ProductAuctionContract, ());
    let client = ProductAuctionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    if mock_auths {
        env.mock_all_auths();
    }

    client.initialize(&admin);

    (env, client, admin, user)
}

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let contract_id = env.register(ProductAuctionContract, ());
    let client = ProductAuctionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    let result = client.try_initialize(&admin);
    assert!(result.is_err());

    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_add_product() {
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &100u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &1000u64;

    let product_id = client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);

    let key = DataKeys::Product(seller.clone(), product_id);
    
    env.as_contract(&client.address, || {
        let stored_product: Product = env
        .storage()
        .persistent()
        .get(&key)
        .expect("Product not found in storage");
        assert_eq!(stored_product.id, product_id);
        assert_eq!(stored_product.seller, seller);
        assert_eq!(stored_product.name, *name);
        assert_eq!(stored_product.description, *description);
        assert_eq!(stored_product.price, *price);
        assert_eq!(stored_product.condition, *condition);
        assert_eq!(stored_product.stock, *stock);
        assert_eq!(stored_product.images, *images);
        assert_eq!(stored_product.weight_grams, *weight_grams);
        assert_eq!(stored_product.verified, false);
    });
}

#[test]
fn test_get_product(){
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &100u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &1000u64;

    let product_id = client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);

    
    let product = client.get_product(&seller, &product_id);

    assert_eq!(product.id, product_id);
    assert_eq!(product.seller, seller);
    assert_eq!(product.name, *name);
    assert_eq!(product.description, *description);
    assert_eq!(product.price, *price);
    assert_eq!(product.condition, *condition);
    assert_eq!(product.stock, *stock);
    assert_eq!(product.images, *images);
    assert_eq!(product.weight_grams, *weight_grams);
    assert_eq!(product.verified, false);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_get_product_invalid_product(){
    let (_, client, _, seller) = setup_test(true);

    let product_id = 1u128;

    client.get_product(&seller, &product_id);
}

#[test]
fn test_get_products() {
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &100u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &1000u64;

    client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);
    client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);

    let products = client.get_products(&seller);

    assert_eq!(products.len(), 2);
}


#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_add_product_unauthorized() {
    let (env, client, _, seller) = setup_test(false);

    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &100u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &1000u64;

    client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_add_product_invalid_description() {
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"Short");
    let price = &100u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &1000u64;

    client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_add_product_invalid_price() {
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &0u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &1000u64;

    client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_add_product_invalid_image() {
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &5u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env];
    let weight_grams = &1000u64;

    client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_add_product_invalid_weight() {
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &5u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &0u64;

    client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);
}

#[test]
fn test_update_stock() {
    let (env, client, _, seller) = setup_test(true);
    let name = &Symbol::new(&env, "Product1");
    let description = &String::from_str(&env,"This is a product");
    let price = &100u64;
    let condition = &Condition::New;
    let stock = &10u32;
    let images = &vec![&env, String::from_str(&env, "image1")];
    let weight_grams = &1000u64;

    let product_id = client.add_product(&seller, name, description, price, condition, stock, images, weight_grams);

    let new_stock = &20u32;
    client.update_stock(&seller, &product_id, new_stock);

    let key = DataKeys::Product(seller.clone(), product_id);
    
    env.as_contract(&client.address, || {
        let stored_product: Product = env
        .storage()
        .persistent()
        .get(&key)
        .expect("Product not found in storage");
        assert_eq!(stored_product.stock, *new_stock);
    });
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_update_stock_invalid_product() {
    let (_, client, _, seller) = setup_test(true);

    let product_id = 1u128;

    let new_stock = &20u32;
    client.update_stock(&seller, &product_id, new_stock);
}





