use soroban_sdk::{contractimpl, Address, Env, Symbol};

use crate::{
    AgriculturalAuctionContract, AgriculturalAuctionContractArgs,
    AgriculturalAuctionContractClient, AgriculturalProduct, DataKey, FreshnessRating, ProductError,
    TimeError,
};

pub trait TimeManagement {
    fn update_product_freshness(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<FreshnessRating, ProductError>;

    fn check_product_expiry(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<bool, ProductError>;

    fn extend_expiry_date(
        env: Env,
        farmer: Address,
        product_id: u64,
        extension_days: u32,
    ) -> Result<u64, TimeError>;

    fn calculate_time_based_price(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<u64, ProductError>;
}

#[contractimpl]
impl TimeManagement for AgriculturalAuctionContract {
    fn update_product_freshness(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<FreshnessRating, ProductError> {
        // Get product details
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let mut product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(ProductError::ProductNotFound)?;

        // Get current time
        let current_time = env.ledger().timestamp();

        // Calculate new freshness rating based on time since harvest
        let age_days = (current_time.saturating_sub(product.harvest_date)) / (24 * 60 * 60);

        let new_freshness = match age_days {
            0..=2 => FreshnessRating::Premium,
            3..=5 => FreshnessRating::Excellent,
            6..=10 => FreshnessRating::Good,
            11..=15 => FreshnessRating::Fair,
            16..=30 => FreshnessRating::Poor,
            _ => FreshnessRating::Expired,
        };

        // Update product freshness
        product.freshness_rating = new_freshness.clone();

        // Adjust price based on freshness
        product.current_price = adjust_price_by_freshness(product.base_price, &new_freshness);

        // Save updated product
        env.storage().persistent().set(&product_key, &product);

        // Emit event for freshness update
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "FreshnessUpdated"),
                product_id,
            ),
            new_freshness.clone(),
        );

        Ok(new_freshness)
    }

    fn check_product_expiry(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<bool, ProductError> {
        // Get product details
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(ProductError::ProductNotFound)?;

        // Get current time
        let current_time = env.ledger().timestamp();

        // Check if product has expired
        let is_expired = product.expiry_date <= current_time;

        Ok(is_expired)
    }

    fn extend_expiry_date(
        env: Env,
        farmer: Address,
        product_id: u64,
        extension_days: u32,
    ) -> Result<u64, TimeError> {
        farmer.require_auth();

        // Get product details
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let mut product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(ProductError::ProductNotFound)
            .map_err(|_| TimeError::InvalidTimeframe)?;

        // Check if product has already expired
        let current_time = env.ledger().timestamp();
        if product.expiry_date <= current_time {
            return Err(TimeError::ProductExpired);
        }

        // Calculate extension in seconds
        let extension_seconds = extension_days as u64 * 24 * 60 * 60;

        // Add extension to current expiry date
        let new_expiry_date = product.expiry_date + extension_seconds;

        // Update product expiry date
        product.expiry_date = new_expiry_date;

        // Save updated product
        env.storage().persistent().set(&product_key, &product);

        // Emit event for expiry extension
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "ExpiryExtended"),
                product_id,
            ),
            new_expiry_date,
        );

        Ok(new_expiry_date)
    }

    fn calculate_time_based_price(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<u64, ProductError> {
        // Get product details
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let mut product: AgriculturalProduct = env
            .storage()
            .persistent()
            .get(&product_key)
            .ok_or(ProductError::ProductNotFound)?;

        // Get current time
        let current_time = env.ledger().timestamp();

        // Calculate remaining shelf life as a percentage
        let total_shelf_life = product.expiry_date.saturating_sub(product.harvest_date);
        let remaining_shelf_life = product.expiry_date.saturating_sub(current_time);

        let remaining_percentage = if total_shelf_life == 0 {
            0
        } else {
            (remaining_shelf_life * 100) / total_shelf_life
        };

        // Adjust price based on remaining shelf life
        let adjusted_price = if remaining_percentage >= 80 {
            // Fresh product, premium price
            product.base_price + (product.base_price * 10 / 100)
        } else if remaining_percentage >= 60 {
            // Good freshness, standard price
            product.base_price
        } else if remaining_percentage >= 40 {
            // Moderate freshness, small discount
            product.base_price - (product.base_price * 10 / 100)
        } else if remaining_percentage >= 20 {
            // Limited freshness, larger discount
            product.base_price - (product.base_price * 25 / 100)
        } else {
            // Near expiry, significant discount
            product.base_price - (product.base_price * 50 / 100)
        };

        // Update product price
        product.current_price = adjusted_price;

        // Save updated product
        env.storage().persistent().set(&product_key, &product);

        // Emit event for price update
        env.events().publish(
            (
                farmer.clone(),
                Symbol::new(&env, "PriceUpdated"),
                product_id,
            ),
            adjusted_price,
        );

        Ok(adjusted_price)
    }
}

// Helper function to adjust price based on freshness
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