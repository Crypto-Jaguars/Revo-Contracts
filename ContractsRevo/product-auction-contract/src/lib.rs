#![no_std]
use datatype::{AdminError, Auction, AuctionError, DataKeys, Product, ProductError, Shipment, ShippingError};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

mod product_auction;
mod listing;
mod shipping;
mod verification;
mod datatype;
mod interfaces;

#[cfg(test)]
mod test;

#[contract]
pub struct ProductAuctionContract;

#[contractimpl]
impl ProductAuctionContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
        if env.storage().instance().has(&DataKeys::Admin) {
            return Err(AdminError::AlreadyVerified);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKeys::Admin, &admin);

        env.events().publish(
            (Symbol::new(&env, "contract_initialized"), admin.clone()),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, AdminError> {
        env.storage()
            .instance()
            .get(&DataKeys::Admin)
            .ok_or(AdminError::UnauthorizedAccess)
    }

    pub fn get_auction(env: Env, seller: Address, product_id: u128) -> Result<Auction, AuctionError> {
        let key = &DataKeys::Auction(seller.clone(), product_id);
        env.storage().instance().get(key).ok_or(AuctionError::AuctionNotFound)
    }

    pub fn get_products(env: Env, seller: Address) -> Result<Vec<Product>, ProductError> {
        let key = DataKeys::ProductList(seller.clone());

        let products = env
            .storage()
            .persistent()
            .get::<_, Vec<Product>>(&key)
            .unwrap_or_else(||
                Vec::new(&env),
            );
        
        return Ok(products);
    }

    pub fn get_product(env: Env, seller: Address, product_id: u128) -> Result<Product, ProductError> {
        let key = DataKeys::Product(seller.clone(), product_id);

        let product = env.storage()
            .persistent()
            .get(&key)
            .ok_or(ProductError::ProductNotFound)?;

        Ok(product)
    }
    
    pub fn get_shipment(env: Env, seller: Address, tracking_number: String) -> Result<Shipment, ShippingError> {
        let shipment_key = DataKeys::Shipment(seller, tracking_number);
        env.storage().persistent().get(&shipment_key).ok_or(ShippingError::ShipmentNotFound)
    }

    pub fn get_shipments(env: Env, seller: Address) -> Result<Vec<Shipment>, ShippingError> {
        let key = DataKeys::ShipmentList(seller.clone());

        let shipments = env
            .storage()
            .persistent()
            .get::<_, Vec<Shipment>>(&key)
            .unwrap_or_else(||
                Vec::new(&env),
            );
        
        return Ok(shipments);
    }
}
