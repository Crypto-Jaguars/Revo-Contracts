use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

use crate::datatypes::{DataKey, Product, Stage, SupplyChainError};

/// Add a new stage to the product lifecycle
/// MANDATORY from roadmap.md
pub fn add_stage(
    env: Env,
    product_id: BytesN<32>,
    stage_name: String,
    location: String,
    handler: Address,
    data_hash: BytesN<32>,
) -> Result<u32, SupplyChainError> {
    handler.require_auth();

    // Validate input data
    if stage_name.len() == 0 || location.len() == 0 {
        return Err(SupplyChainError::InvalidInput);
    }

    // Get existing product
    let mut product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
        .ok_or(SupplyChainError::ProductNotFound)?;

    // Generate new stage ID
    let stage_id = product.stages.len() as u32 + 1;

    // Create new stage
    let stage = Stage {
        stage_id,
        name: stage_name.clone(),
        timestamp: env.ledger().timestamp(),
        location: location.clone(),
        data_hash,
    };

    // Add stage to product's stages vector
    product.stages.push_back(stage.clone());

    // Store updated product (with new stage embedded)
    env.storage()
        .persistent()
        .set(&DataKey::Product(product_id.clone()), &product);

    // Emit event
    env.events().publish(
        (Symbol::new(&env, "stage_added"), handler),
        (product_id, stage_id),
    );

    Ok(stage_id)
}

/// Get the full product trace including all stages
/// MANDATORY from roadmap.md
pub fn get_product_trace(
    env: Env,
    product_id: BytesN<32>,
) -> Result<(Product, Vec<Stage>), SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    // Stages are already embedded in product, just clone them
    let stages = product.stages.clone();

    Ok((product, stages))
}

/// Get the current stage of a product
/// EXTENDED functionality
pub fn get_current_stage(
    env: Env,
    product_id: BytesN<32>,
) -> Result<Stage, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    if product.stages.is_empty() {
        return Err(SupplyChainError::StageNotFound);
    }

    // Get the last stage (most recent)
    let last_index = product.stages.len() - 1;
    let current_stage = product.stages.get(last_index).unwrap();

    Ok(current_stage)
}

/// Get complete stage history for a product
/// EXTENDED functionality
pub fn get_stage_history(
    env: Env,
    product_id: BytesN<32>,
) -> Result<Vec<Stage>, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    Ok(product.stages)
}

/// Validate stage transition logic
/// EXTENDED functionality
pub fn validate_stage_transition(
    env: Env,
    product_id: BytesN<32>,
    from_stage: u32,
    to_stage: u32,
) -> Result<bool, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    // Basic validation: to_stage should be sequential
    if to_stage != from_stage + 1 {
        return Ok(false);
    }

    // Check if from_stage exists in product
    let stage_exists = product.stages.iter().any(|stage| stage.stage_id == from_stage);
    if !stage_exists {
        return Err(SupplyChainError::StageNotFound);
    }

    // Check for duplicate stage
    let duplicate_exists = product.stages.iter().any(|stage| stage.stage_id == to_stage);
    if duplicate_exists {
        return Err(SupplyChainError::DuplicateStage);
    }

    Ok(true)
}

/// Get a specific stage by ID
/// EXTENDED functionality
pub fn get_stage_by_id(
    env: Env,
    product_id: BytesN<32>,
    stage_id: u32,
) -> Result<Stage, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    for stage in product.stages.iter() {
        if stage.stage_id == stage_id {
            return Ok(stage);
        }
    }

    Err(SupplyChainError::StageNotFound)
}