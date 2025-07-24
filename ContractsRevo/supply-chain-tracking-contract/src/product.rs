use crate::datatypes::{CertificateId, DataKey, Product, ProductRegistration, SupplyChainError};
use crate::utils;
use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

/// Register a new agricultural product with initial details
pub fn register_product(
    env: Env,
    farmer_id: Address,
    product_type: String,
    batch_number: String,
    origin_location: String,
    metadata_hash: BytesN<32>,
) -> Result<BytesN<32>, SupplyChainError> {
    farmer_id.require_auth();

    // Validate input data
    if product_type.is_empty() || batch_number.is_empty() || origin_location.is_empty() {
        return Err(SupplyChainError::InvalidInput);
    }

    // Generate unique product ID
    let product_id = utils::generate_product_id(&env, &farmer_id, &product_type, &batch_number);

    // Check if product already exists
    if env
        .storage()
        .persistent()
        .has(&DataKey::Product(product_id.clone()))
    {
        return Err(SupplyChainError::InvalidProductData);
    }

    // Create product with EMPTY stages vector initially
    let product = Product {
        product_id: product_id.clone(),
        farmer_id: farmer_id.clone(),
        stages: Vec::new(&env),
        certificate_id: CertificateId::None,
    };

    // Store product
    env.storage()
        .persistent()
        .set(&DataKey::Product(product_id.clone()), &product);

    // Create ProductRegistration struct to store all registration details
    let registration = ProductRegistration {
        product_type: product_type.clone(),
        batch_number: batch_number.clone(),
        origin_location: origin_location.clone(),
        metadata_hash: metadata_hash.clone(),
    };

    // Store the registration details
    env.storage().persistent().set(
        &DataKey::ProductRegistration(product_id.clone()),
        &registration,
    );

    // Update farmer's product list
    update_farmer_products(&env, &farmer_id, &product_id)?;

    // Update product type index for traceability
    update_product_type_index(&env, &product_type, &product_id)?;

    // Emit event
    env.events().publish(
        (Symbol::new(&env, "product_registered"), farmer_id),
        product_id.clone(),
    );

    Ok(product_id)
}

/// Get product registration details
pub fn get_product_registration(
    env: Env,
    product_id: BytesN<32>,
) -> Result<ProductRegistration, SupplyChainError> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductRegistration(product_id))
        .ok_or(SupplyChainError::ProductNotFound)
}

/// Get product details
pub fn get_product_details(env: Env, product_id: BytesN<32>) -> Result<Product, SupplyChainError> {
    env.storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)
}

/// List all products for a specific farmer
pub fn list_products_by_farmer(
    env: Env,
    farmer_id: Address,
) -> Result<Vec<BytesN<32>>, SupplyChainError> {
    let products = env
        .storage()
        .persistent()
        .get(&DataKey::FarmerProducts(farmer_id))
        .unwrap_or_else(|| Vec::new(&env));

    Ok(products)
}

/// List products by product type for traceability
pub fn list_products_by_type(
    env: Env,
    product_type: String,
) -> Result<Vec<BytesN<32>>, SupplyChainError> {
    let products = env
        .storage()
        .persistent()
        .get(&DataKey::ProductTypeIndex(product_type))
        .unwrap_or_else(|| Vec::new(&env));

    Ok(products)
}

/// Helper function to update farmer's product list
fn update_farmer_products(
    env: &Env,
    farmer_id: &Address,
    product_id: &BytesN<32>,
) -> Result<(), SupplyChainError> {
    let key = DataKey::FarmerProducts(farmer_id.clone());
    let mut products: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    products.push_back(product_id.clone());
    env.storage().persistent().set(&key, &products);

    env.events().publish(
        (Symbol::new(env, "farmer_products_updated"),),
        (farmer_id.clone(), product_id.clone()),
    );

    Ok(())
}

/// Helper function to update product type index
fn update_product_type_index(
    env: &Env,
    product_type: &String,
    product_id: &BytesN<32>,
) -> Result<(), SupplyChainError> {
    let key = DataKey::ProductTypeIndex(product_type.clone());
    let mut products: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    products.push_back(product_id.clone());
    env.storage().persistent().set(&key, &products);

    Ok(())
}
