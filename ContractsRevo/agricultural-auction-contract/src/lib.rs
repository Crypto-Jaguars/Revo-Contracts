#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

mod auction_core;
mod datatype;
mod price_oracle;
mod product_listing;
mod time_management;
mod test;

pub use datatype::*;

#[contract]
pub struct AgriculturalAuctionContract;

#[contractimpl]
impl AgriculturalAuctionContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AdminError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        env.events().publish(
            (Symbol::new(&env, "contract_initialized"), admin.clone()),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, AdminError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AdminError::UnauthorizedAccess)
    }

    pub fn get_auction(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<Auction, AuctionError> {
        let key = &DataKey::Auction(farmer.clone(), product_id);
        env.storage()
            .instance()
            .get(key)
            .ok_or(AuctionError::AuctionNotFound)
    }

    pub fn get_products(
        env: Env,
        farmer: Address,
    ) -> Result<Vec<AgriculturalProduct>, ProductError> {
        let key = DataKey::ProductList(farmer.clone());

        let products = env
            .storage()
            .persistent()
            .get::<_, Vec<AgriculturalProduct>>(&key)
            .unwrap_or_else(|| Vec::new(&env));

        Ok(products)
    }

    pub fn get_product(
        env: Env,
        farmer: Address,
        product_id: u64,
    ) -> Result<AgriculturalProduct, ProductError> {
        let key = DataKey::Product(farmer.clone(), product_id);

        let product = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ProductError::ProductNotFound)?;

        Ok(product)
    }

    pub fn get_market_price(
        env: Env,
        product_type: Symbol,
        region: Symbol,
    ) -> Result<u64, OracleError> {
        let key = DataKey::MarketPrice(product_type, region);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(OracleError::PriceDataNotAvailable)
    }

    pub fn get_seasonal_status(
        env: Env,
        product_type: Symbol,
        region: Symbol,
    ) -> Result<SeasonalStatus, ProductError> {
        let key = DataKey::SeasonalStatus(product_type, region);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(ProductError::SeasonalDataNotAvailable)
    }
}
