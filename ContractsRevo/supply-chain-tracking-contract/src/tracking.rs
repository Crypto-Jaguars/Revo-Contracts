use crate::datatypes::{DataKey, Product, Stage, StageTier, SupplyChainError};
use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

/// Add a new stage to the product lifecycle with tier validation
pub fn add_stage(
    env: Env,
    product_id: BytesN<32>,
    stage_tier: StageTier,
    stage_name: String,
    location: String,
    handler: Address,
    data_hash: BytesN<32>,
) -> Result<u32, SupplyChainError> {
    handler.require_auth();

    // Validate input data
    if stage_name.is_empty() || location.is_empty() {
        return Err(SupplyChainError::InvalidInput);
    }

    // Get existing product
    let mut product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
        .ok_or(SupplyChainError::ProductNotFound)?;

    // Validate tier progression
    validate_tier_progression(&product, &stage_tier)?;

    // Generate new stage ID
    let stage_id = product.stages.len() + 1;

    // Create new stage
    let stage = Stage {
        stage_id,
        tier: stage_tier,
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
pub fn get_product_trace(
    env: Env,
    product_id: BytesN<32>,
) -> Result<(Product, Vec<Stage>), SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    let stages = product.stages.clone();

    Ok((product, stages))
}

/// Get the current stage of a product
pub fn get_current_stage(env: Env, product_id: BytesN<32>) -> Result<Stage, SupplyChainError> {
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
pub fn get_stage_history(env: Env, product_id: BytesN<32>) -> Result<Vec<Stage>, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    Ok(product.stages)
}

/// Validate stage transition logic
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
    // This validation assumes stages are never deleted and IDs remain sequential
    if to_stage != from_stage + 1 {
        return Ok(false);
    }

    // Check if from_stage exists in product
    let stage_exists = product
        .stages
        .iter()
        .any(|stage| stage.stage_id == from_stage);
    if !stage_exists {
        return Err(SupplyChainError::StageNotFound);
    }

    // Check for duplicate stage
    let duplicate_exists = product
        .stages
        .iter()
        .any(|stage| stage.stage_id == to_stage);
    if duplicate_exists {
        return Err(SupplyChainError::DuplicateStage);
    }

    Ok(true)
}

/// Validate tier progression logic
fn validate_tier_progression(
    product: &Product,
    new_tier: &StageTier,
) -> Result<(), SupplyChainError> {
    // Check for duplicate tier
    for existing_stage in product.stages.iter() {
        if existing_stage.tier == *new_tier {
            return Err(SupplyChainError::DuplicateStageTier);
        }
    }

    // If no stages exist, must start with Planting
    if product.stages.is_empty() {
        if *new_tier != StageTier::Planting {
            return Err(SupplyChainError::InvalidTierProgression);
        }
        return Ok(());
    }

    // Get the current (last) stage tier
    let current_stage = product.stages.get(product.stages.len() - 1).unwrap();
    let current_tier = &current_stage.tier;

    // Check if new tier is the next expected tier
    match current_tier.next() {
        Some(expected_next_tier) => {
            if *new_tier != expected_next_tier {
                return Err(SupplyChainError::InvalidTierProgression);
            }
        }
        None => {
            // Current tier is Consumer (final stage), no more stages allowed
            return Err(SupplyChainError::InvalidTierProgression);
        }
    }

    Ok(())
}

/// Get the next expected tier for a product
pub fn get_next_expected_tier(
    env: Env,
    product_id: BytesN<32>,
) -> Result<Option<StageTier>, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    if product.stages.is_empty() {
        return Ok(Some(StageTier::Planting));
    }

    let current_stage = product.stages.get(product.stages.len() - 1).unwrap();
    Ok(current_stage.tier.next())
}

/// Get the current tier for a product
pub fn get_current_tier(
    env: Env,
    product_id: BytesN<32>,
) -> Result<Option<StageTier>, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    if product.stages.is_empty() {
        return Ok(None);
    }

    let current_stage = product.stages.get(product.stages.len() - 1).unwrap();
    Ok(Some(current_stage.tier.clone()))
}

/// Get a specific stage by ID
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
