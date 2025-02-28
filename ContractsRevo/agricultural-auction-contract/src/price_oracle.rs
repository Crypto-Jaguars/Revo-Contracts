use soroban_sdk::{contractimpl, Address, Env, Symbol, Vec};

use crate::{
    AgriculturalAuctionContract, AgriculturalAuctionContractArgs,
    AgriculturalAuctionContractClient, AgriculturalProduct, DataKey, MarketPrice, OracleError,
    ProductError, SeasonalStatus,
};

pub trait PriceOracle {
    fn update_market_price(
        env: Env,
        admin: Address,
        product_type: Symbol,
        region: Symbol,
        price: u64,
        trend: i32,
        volume: u64,
    ) -> Result<(), OracleError>;

    fn fetch_market_price(
        env: Env,
        product_type: Symbol,
        region: Symbol,
    ) -> Result<MarketPrice, OracleError>;

    fn compare_with_market(env: Env, farmer: Address, product_id: u64)
        -> Result<i32, ProductError>;

    fn suggest_price(
        env: Env,
        product_type: Symbol,
        region: Symbol,
        quality_grade: Symbol,
        freshness_rating: Symbol,
    ) -> Result<u64, OracleError>;

    fn update_regional_prices(
        env: Env,
        admin: Address,
        product_type: Symbol,
        regions: Vec<Symbol>,
        prices: Vec<u64>,
    ) -> Result<(), OracleError>;
}

#[contractimpl]
impl PriceOracle for AgriculturalAuctionContract {
    fn update_market_price(
        env: Env,
        admin: Address,
        product_type: Symbol,
        region: Symbol,
        price: u64,
        trend: i32,
        volume: u64,
    ) -> Result<(), OracleError> {
        // Ensure admin authorization
        admin.require_auth();

        // Verify admin is authorized
        let stored_admin = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .ok_or(OracleError::InvalidPriceData)?;

        if admin != stored_admin {
            return Err(OracleError::InvalidPriceData);
        }

        // Create market price data
        let market_price = MarketPrice {
            product_type: product_type.clone(),
            region: region.clone(),
            price,
            timestamp: env.ledger().timestamp(),
            trend,
            volume,
        };

        // Store current market price
        let key = DataKey::MarketPrice(product_type.clone(), region.clone());
        env.storage().persistent().set(&key, &market_price);

        // Also store in price history
        let history_key = DataKey::PriceHistory(
            product_type.clone(),
            region.clone(),
            env.ledger().timestamp(),
        );
        env.storage().persistent().set(&history_key, &price);

        // Emit event for price update
        env.events().publish(
            (
                Symbol::new(&env, "MarketPriceUpdated"),
                product_type,
                region,
            ),
            price,
        );

        Ok(())
    }

    fn fetch_market_price(
        env: Env,
        product_type: Symbol,
        region: Symbol,
    ) -> Result<MarketPrice, OracleError> {
        let key = DataKey::MarketPrice(product_type, region);

        env.storage()
            .persistent()
            .get(&key)
            .ok_or(OracleError::PriceDataNotAvailable)
    }

    fn compare_with_market(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<i32, ProductError> {
        // Get product details
        let product_key = DataKey::Product(farmer.clone(), product_id);
        let product = env
            .storage()
            .persistent()
            .get::<_, AgriculturalProduct>(&product_key)
            .ok_or(ProductError::ProductNotFound)?;

        // Get market price
        let market_key = DataKey::MarketPrice(product.product_type.clone(), product.region.clone());
        let market_price = env
            .storage()
            .persistent()
            .get::<_, MarketPrice>(&market_key)
            .ok_or(OracleError::PriceDataNotAvailable)
            .map_err(|_| ProductError::SeasonalDataNotAvailable)?;

        // Calculate difference as percentage
        // Positive: product is more expensive than market
        // Negative: product is cheaper than market
        let difference = if market_price.price == 0 {
            0
        } else {
            ((product.current_price as i64 - market_price.price as i64) * 100) as i32
                / market_price.price as i32
        };

        Ok(difference)
    }

    fn suggest_price(
        env: Env,
        product_type: Symbol,
        region: Symbol,
        quality_grade: Symbol,
        freshness_rating: Symbol,
    ) -> Result<u64, OracleError> {
        // Get market price
        let market_key = DataKey::MarketPrice(product_type.clone(), region.clone());
        let market_price = env
            .storage()
            .persistent()
            .get::<_, MarketPrice>(&market_key)
            .ok_or(OracleError::PriceDataNotAvailable)?;

        // Get seasonal status
        let seasonal_key = DataKey::SeasonalStatus(product_type.clone(), region.clone());
        let seasonal_status = env
            .storage()
            .persistent()
            .get::<_, SeasonalStatus>(&seasonal_key)
            .unwrap_or(SeasonalStatus::YearRound);

        // Base price from market data
        let base_price = market_price.price;

        // Define constants for quality grades
        let premium = Symbol::new(&env, "Premium");
        let grade_a = Symbol::new(&env, "Grade_A");
        let grade_b = Symbol::new(&env, "Grade_B");
        let grade_c = Symbol::new(&env, "Grade_C");
        let substandard = Symbol::new(&env, "Substandard");
        let rejected = Symbol::new(&env, "Rejected");

        // Adjust for quality
        let quality_adjusted = if quality_grade == premium {
            base_price + (base_price * 30 / 100) // +30%
        } else if quality_grade == grade_a {
            base_price + (base_price * 15 / 100) // +15%
        } else if quality_grade == grade_b {
            base_price // No change
        } else if quality_grade == grade_c {
            base_price - (base_price * 15 / 100) // -15%
        } else if quality_grade == substandard {
            base_price - (base_price * 30 / 100) // -30%
        } else if quality_grade == rejected {
            base_price - (base_price * 80 / 100) // -80%
        } else {
            base_price
        };

        // Define constants for freshness ratings
        let freshness_premium = Symbol::new(&env, "Premium");
        let freshness_excellent = Symbol::new(&env, "Excellent");
        let freshness_good = Symbol::new(&env, "Good");
        let freshness_fair = Symbol::new(&env, "Fair");
        let freshness_poor = Symbol::new(&env, "Poor");
        let freshness_expired = Symbol::new(&env, "Expired");

        // Adjust for freshness
        let freshness_adjusted = if freshness_rating == freshness_premium {
            quality_adjusted + (quality_adjusted * 20 / 100) // +20%
        } else if freshness_rating == freshness_excellent {
            quality_adjusted + (quality_adjusted * 10 / 100) // +10%
        } else if freshness_rating == freshness_good {
            quality_adjusted // No change
        } else if freshness_rating == freshness_fair {
            quality_adjusted - (quality_adjusted * 10 / 100) // -10%
        } else if freshness_rating == freshness_poor {
            quality_adjusted - (quality_adjusted * 25 / 100) // -25%
        } else if freshness_rating == freshness_expired {
            quality_adjusted - (quality_adjusted * 50 / 100) // -50%
        } else {
            quality_adjusted
        };

        // Adjust for seasonal status
        let final_price = match seasonal_status {
            SeasonalStatus::InSeason => freshness_adjusted - (freshness_adjusted * 5 / 100), // -5% (abundance)
            SeasonalStatus::EarlySeason => freshness_adjusted + (freshness_adjusted * 10 / 100), // +10% (novelty)
            SeasonalStatus::LateSeason => freshness_adjusted - (freshness_adjusted * 10 / 100), // -10% (clearing stock)
            SeasonalStatus::OutOfSeason => freshness_adjusted + (freshness_adjusted * 30 / 100), // +30% (scarcity)
            SeasonalStatus::YearRound => freshness_adjusted,
        };

        // Adjust for market trend
        let trend_adjusted =
            final_price as i64 + ((final_price as i64 * market_price.trend as i64) / 100);

        // Ensure non-negative price
        let suggested_price = if trend_adjusted < 0 {
            0
        } else {
            trend_adjusted as u64
        };

        Ok(suggested_price)
    }

    fn update_regional_prices(
        env: Env,
        admin: Address,
        product_type: Symbol,
        regions: Vec<Symbol>,
        prices: Vec<u64>,
    ) -> Result<(), OracleError> {
        // Ensure admin authorization
        admin.require_auth();

        // Verify admin is authorized
        let stored_admin = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .ok_or(OracleError::InvalidPriceData)?;

        if admin != stored_admin {
            return Err(OracleError::InvalidPriceData);
        }

        // Ensure regions and prices vectors have same length
        if regions.len() != prices.len() {
            return Err(OracleError::InvalidPriceData);
        }

        let current_time = env.ledger().timestamp();

        // Update prices for each region
        for i in 0..regions.len() {
            let region = regions.get(i).unwrap();
            let price = prices.get(i).unwrap();

            // Get existing market price to maintain trend and volume data
            let key = DataKey::MarketPrice(product_type.clone(), region.clone());
            let existing_market_price = env.storage().persistent().get::<_, MarketPrice>(&key);

            // Default values for trend and volume if no existing data
            let (trend, volume) = match existing_market_price {
                Some(mp) => (mp.trend, mp.volume),
                None => (0, 0),
            };

            // Create updated market price
            let market_price = MarketPrice {
                product_type: product_type.clone(),
                region: region.clone(),
                price,
                timestamp: current_time,
                trend,
                volume,
            };

            // Save updated market price
            env.storage().persistent().set(&key, &market_price);

            // Also store in price history
            let history_key =
                DataKey::PriceHistory(product_type.clone(), region.clone(), current_time);
            env.storage().persistent().set(&history_key, &price);
        }

        // Emit event for regional price updates
        env.events().publish(
            (Symbol::new(&env, "RegionalPricesUpdated"), product_type),
            regions.len(),
        );

        Ok(())
    }
}