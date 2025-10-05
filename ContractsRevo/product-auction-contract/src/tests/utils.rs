use crate::datatype::Condition;
use crate::ProductAuctionContract;
use crate::ProductAuctionContractClient;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String, Symbol};

pub fn setup_env() -> Env {
    Env::default()
}

pub fn setup_contract(env: &'_ Env) -> ProductAuctionContractClient<'_> {
    let contract_id = env.register(ProductAuctionContract, ());
    ProductAuctionContractClient::new(env, &contract_id)
}

pub fn setup_with_admin(
    env: &Env,
    client: &ProductAuctionContractClient,
    mock_auth: bool,
) -> Address {
    let admin = Address::generate(env);
    if mock_auth {
        env.mock_all_auths();
    }
    client.initialize(&admin);
    admin
}

pub fn create_test_product(
    env: &Env,
    client: &ProductAuctionContractClient,
    seller: &Address,
) -> u64 {
    let name = Symbol::new(env, "TestProduct");
    let description = String::from_str(env, "Test product description");
    let price = 100u64;
    let condition = Condition::New;
    let stock = 10u32;
    let images = vec![env, String::from_str(env, "image1.jpg")];
    let weight_pounds = 5u64;

    client.add_product(
        seller,
        &name,
        &description,
        &price,
        &condition,
        &stock,
        &images,
        &weight_pounds,
    )
}

pub fn create_auction_with_product(
    env: &Env,
    client: &ProductAuctionContractClient,
    seller: &Address,
    end_time_offset: u64,
) -> u64 {
    let product_id = create_test_product(env, client, seller);
    let reserve_price = 50u64;
    let auction_end_time = env.ledger().timestamp() + end_time_offset;

    client.create_auction(seller, &reserve_price, &auction_end_time, &product_id);
    product_id
}
