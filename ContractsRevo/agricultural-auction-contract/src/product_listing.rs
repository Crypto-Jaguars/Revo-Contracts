use soroban_sdk::{contractimpl, contracttype, Address, Env, String, Symbol, Vec};

use crate::{
    AgriculturalAuctionContract, AgriculturalAuctionContractArgs,
    AgriculturalAuctionContractClient, AgriculturalProduct, DataKey, FreshnessRating, ProductError,
    QualityGrade, SeasonalStatus, StorageCondition, TimeError,
};

pub trait ProductListing {
    fn add_product(
        env: Env,
        farmer: Address,
        product_details: ProductDetails,
    ) -> Result<u64, ProductError>;

    fn update_freshness(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_rating: FreshnessRating,
    ) -> Result<(), ProductError>;

    fn update_quantity(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_quantity: u32,
    ) -> Result<(), ProductError>;

    fn verify_seasonal_status(
        env: Env,
        product_type: Symbol,
        region: Symbol,
    ) -> Result<SeasonalStatus, ProductError>;

    fn calculate_expiry_date(
        env: Env,
        harvest_date: u64,
        product_type: Symbol,
    ) -> Result<u64, TimeError>;

    fn update_quality_grade(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_grade: QualityGrade,
    ) -> Result<(), ProductError>;
}

#[contracttype]
#[derive(Clone)]
pub struct ProductDetails {
    pub name: Symbol,
    pub description: String,
    pub base_price: u64,
    pub weight_kg: u64,
    pub quantity: u32,
    pub harvest_date: u64,
    pub images: Vec<String>,
    pub certifications: Vec<Symbol>,
    pub storage_condition: StorageCondition,
    pub product_type: Symbol,
    pub region: Symbol,
}

#[contractimpl]
impl ProductListing for AgriculturalAuctionContract {
    fn add_product(
        env: Env,
        farmer: Address,
        product_details: ProductDetails,
    ) -> Result<u64, ProductError> {
        farmer.require_auth();

        // Validate description length
        if product_details.description.len() < 10 || product_details.description.len() > 500 {
            return Err(ProductError::InvalidDescription);
        }

        // Validate price is not zero
        if product_details.base_price == 0 {
            return Err(ProductError::InvalidPrice);
        }

        // Ensure there is at least one image
        if product_details.images.is_empty() || product_details.images.len() > 10 {
            return Err(ProductError::InvalidImageCount);
        }

        // Validate product weight
        if product_details.weight_kg == 0 {
            return Err(ProductError::InvalidWeight);
        }

        // Validate harvest date
        let current_time = env.ledger().timestamp();
        if product_details.harvest_date > current_time {
            return Err(ProductError::InvalidHarvestDate);
        }

        // Check seasonal status
        let seasonal_status = Self::verify_seasonal_status(
            env.clone(),
            product_details.product_type.clone(),
            product_details.region.clone(),
        )?;

        if seasonal_status == SeasonalStatus::OutOfSeason {
            return Err(ProductError::OutOfSeason);
        }

        // Calculate expiry date based on product type and harvest date
        let expiry_date = Self::calculate_expiry_date(
            env.clone(),
            product_details.harvest_date,
            product_details.product_type.clone(),
        )
        .map_err(|_e| ProductError::InvalidHarvestDate)?;

        // Default freshness rating based on harvest date
        let freshness_rating = calculate_freshness(product_details.harvest_date, current_time);

        // Generate a unique product ID
        let product_id: u64 = env.prng().gen();

        // Create the product
        let product = AgriculturalProduct {
            id: product_id,
            farmer: farmer.clone(),
            name: product_details.name.clone(),
            description: product_details.description,
            base_price: product_details.base_price,
            current_price: product_details.base_price, // Initially set to base price
            weight_kg: product_details.weight_kg,
            quantity: product_details.quantity,
            harvest_date: product_details.harvest_date,
            expiry_date,
            images: product_details.images,
            freshness_rating,
            quality_grade: QualityGrade::GradeB, // Default grade until verified
            verified: false,
            certifications: product_details.certifications,
            storage_condition: product_details.storage_condition,
            product_type: product_details.product_type.clone(),
            region: product_details.region.clone(),
            seasonal_status,
        };

        // Retrieve or initialize the product list for the farmer
        let key = DataKey::ProductList(farmer.clone());
        let mut products = env
            .storage()
            .persistent()
            .get::<_, Vec<AgriculturalProduct>>(&key)
            .unwrap_or_else(|| Vec::new(&env));

        // Add the new product to the list
        products.push_back(product.clone());

        // Save the updated product list
        env.storage().persistent().set(&key, &products);

        // Save the individual product under its own key
        let product_key = DataKey::Product(farmer.clone(), product_id);
        env.storage().persistent().set(&product_key, &product);

        // Emit an event for the new product
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "ProductAdded"),
                product_details.name.clone(),
            ),
            product.clone(),
        );

        Ok(product_id)
    }

    fn update_freshness(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_rating: FreshnessRating,
    ) -> Result<(), ProductError> {
        farmer.require_auth();

        let product_key = DataKey::Product(farmer.clone(), product_id);
        let mut product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(ProductError::ProductNotFound)?;

        // Update the freshness rating
        product.freshness_rating = new_rating.clone();

        // Adjust current price based on freshness
        product.current_price = adjust_price_by_freshness(product.base_price, &new_rating);

        // Save the updated product
        env.storage().persistent().set(&product_key, &product);

        // Emit event for freshness update
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "FreshnessUpdated"),
                product_id,
            ),
            new_rating,
        );

        Ok(())
    }

    fn update_quantity(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_quantity: u32,
    ) -> Result<(), ProductError> {
        farmer.require_auth();

        let product_key = DataKey::Product(farmer.clone(), product_id);
        let mut product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(ProductError::ProductNotFound)?;

        // Update quantity
        product.quantity = new_quantity;

        // Save the updated product
        env.storage().persistent().set(&product_key, &product);

        // Emit event for quantity update
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "QuantityUpdated"),
                product_id,
            ),
            new_quantity,
        );

        Ok(())
    }

    fn verify_seasonal_status(
        env: Env,
        product_type: Symbol,
        region: Symbol,
    ) -> Result<SeasonalStatus, ProductError> {
        let key = DataKey::SeasonalStatus(product_type.clone(), region.clone());

        // Try to get seasonal status from storage
        if let Some(status) = env.storage().persistent().get::<_, SeasonalStatus>(&key) {
            return Ok(status);
        }

        // This should likely query an oracle or external data source
        // If no data available, assume it's year-round (default)
        let default_status = SeasonalStatus::YearRound;

        // Store this for future reference
        env.storage().persistent().set(&key, &default_status);

        Ok(default_status)
    }

    fn calculate_expiry_date(
        env: Env,
        harvest_date: u64,
        product_type: Symbol,
    ) -> Result<u64, TimeError> {
        let current_time = env.ledger().timestamp();

        if harvest_date > current_time {
            return Err(TimeError::HarvestDateInFuture);
        }

        // Define constants for product types
        let leafy_greens = Symbol::new(&env, "Leafy_Greens");
        let berries = Symbol::new(&env, "Berries");
        let root_vegetables = Symbol::new(&env, "Root_Vegetables");
        let citrus = Symbol::new(&env, "Citrus");
        let grains = Symbol::new(&env, "Grains");

        // Get shelf life in seconds based on product type
        let shelf_life = if product_type == leafy_greens {
            7 * 24 * 60 * 60 // 7 days
        } else if product_type == berries {
            5 * 24 * 60 * 60 // 5 days
        } else if product_type == root_vegetables {
            30 * 24 * 60 * 60 // 30 days
        } else if product_type == citrus {
            21 * 24 * 60 * 60 // 21 days
        } else if product_type == grains {
            180 * 24 * 60 * 60 // 180 days
        } else {
            14 * 24 * 60 * 60 // Default: 14 days
        };

        Ok(harvest_date + shelf_life)
    }

    fn update_quality_grade(
        env: Env,
        farmer: Address,
        product_id: u64,
        new_grade: QualityGrade,
    ) -> Result<(), ProductError> {
        farmer.require_auth();

        let product_key = DataKey::Product(farmer.clone(), product_id);
        let mut product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(ProductError::ProductNotFound)?;

        // Update the quality grade
        product.quality_grade = new_grade.clone();

        // Price adjustment based on quality grade
        product.current_price = adjust_price_by_quality(product.current_price, &new_grade);

        // Save the updated product
        env.storage().persistent().set(&product_key, &product);

        // Emit event for quality update
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "QualityUpdated"),
                product_id,
            ),
            new_grade,
        );

        Ok(())
    }
}

// Helper function to calculate freshness based on harvest date and current time
fn calculate_freshness(harvest_date: u64, current_time: u64) -> FreshnessRating {
    let age_days = (current_time.saturating_sub(harvest_date)) / (24 * 60 * 60);

    match age_days {
        0..=2 => FreshnessRating::Premium,
        3..=5 => FreshnessRating::Excellent,
        6..=10 => FreshnessRating::Good,
        11..=15 => FreshnessRating::Fair,
        16..=30 => FreshnessRating::Poor,
        _ => FreshnessRating::Expired,
    }
}

fn adjust_price_by_freshness(base_price: u64, freshness: &FreshnessRating) -> u64 {
    match freshness {
        FreshnessRating::Premium => base_price + (base_price * 20 / 100), // +20%
        FreshnessRating::Excellent => base_price + (base_price * 10 / 100), // +10%
        FreshnessRating::Good => base_price,                              // Base price
        FreshnessRating::Fair => base_price - (base_price * 10 / 100),    // -10%
        FreshnessRating::Poor => base_price - (base_price * 25 / 100),    // -25%
        FreshnessRating::Expired => base_price - (base_price * 50 / 100), // -50%
    }
}

fn adjust_price_by_quality(current_price: u64, quality: &QualityGrade) -> u64 {
    match quality {
        QualityGrade::Premium => current_price + (current_price * 30 / 100), // +30%
        QualityGrade::GradeA => current_price + (current_price * 15 / 100),  // +15%
        QualityGrade::GradeB => current_price,                               // No change
        QualityGrade::GradeC => current_price - (current_price * 15 / 100),  // -15%
        QualityGrade::Substandard => current_price - (current_price * 30 / 100), // -30%
        QualityGrade::Rejected => current_price - (current_price * 80 / 100), // -80%
    }
}