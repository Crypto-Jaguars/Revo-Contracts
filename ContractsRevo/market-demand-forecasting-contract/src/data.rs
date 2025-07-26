use crate::error::ContractError;
use crate::storage::{self, Product};
use crate::utils::utils;
use soroban_sdk::{BytesN, Env, String, Vec};

/// Handles the logic for managing product data.
pub fn register_product(
    env: &Env,
    name: String,
    historical_demand: Vec<i128>,
) -> Result<BytesN<32>, ContractError> {
    if name.is_empty() {
        return Err(ContractError::InvalidData);
    }

    // Generate a unique ID for the product based on its name and the creation time.
    let product_id = utils::generate_id(env, (name.clone(), env.ledger().timestamp()));

    let product = Product {
        product_id: product_id.clone(),
        name,
        historical_demand,
    };

    // Save the product and add its ID to the global list of all products.
    storage::set_product(env, &product);
    storage::add_product_id(env, &product_id);

    Ok(product_id)
}
