#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String, Vec};

mod barter;
mod error;
mod reputation;
mod trade;
mod utils;

pub use barter::*;
pub use error::*;
pub use reputation::*;
pub use trade::*;
pub use utils::*;

#[cfg(test)]
mod test;

// Data structures for trade offers
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeOffer {
    pub offer_id: BytesN<32>,
    pub cooperative_id: Address,
    pub offered_product: BytesN<32>,
    pub requested_product: BytesN<32>,
    pub status: String, // "Pending", "Accepted", "Completed"
}

// Reputation tracking structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reputation {
    pub cooperative_id: Address,
    pub successful_trades: u32,
    pub rating: u32, // 1-5 scale
}

// Barter agreement structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BarterAgreement {
    pub agreement_id: BytesN<32>,
    pub trade_offer_id: BytesN<32>,
    pub offering_cooperative: Address,
    pub accepting_cooperative: Address,
    pub status: String, // "Active", "Completed", "Disputed"
}

// Data storage keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TradeOffer(BytesN<32>),
    BarterAgreement(BytesN<32>),
    Reputation(Address),
    ActiveOffers,
    OfferCounter,
    AgreementCounter,
}

#[contract]
pub struct CrossCooperativeTradeContract;

#[contractimpl]
impl CrossCooperativeTradeContract {
    /// Initialize the contract with an admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AdminError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::OfferCounter, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::AgreementCounter, &0u32);

        // Initialize active offers list
        let active_offers: Vec<BytesN<32>> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::ActiveOffers, &active_offers);

        Ok(())
    }

    /// Get the contract admin
    pub fn get_admin(env: Env) -> Result<Address, AdminError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AdminError::NotInitialized)
    }

    // Trade Management Functions
    /// Create a new trade offer
    pub fn create_trade_offer(
        env: Env,
        cooperative_id: Address,
        offered_product: BytesN<32>,
        requested_product: BytesN<32>,
    ) -> Result<BytesN<32>, TradeError> {
        trade::create_trade_offer(env, cooperative_id, offered_product, requested_product)
    }

    /// Accept a trade offer
    pub fn accept_trade(
        env: Env,
        offer_id: BytesN<32>,
        accepting_cooperative: Address,
    ) -> Result<BytesN<32>, TradeError> {
        trade::accept_trade(env, offer_id, accepting_cooperative)
    }

    /// Complete a trade
    pub fn complete_trade(
        env: Env,
        offer_id: BytesN<32>,
        caller: Address,
    ) -> Result<(), TradeError> {
        trade::complete_trade(env, offer_id, caller)
    }

    /// Get trade details
    pub fn get_trade_details(env: Env, offer_id: BytesN<32>) -> Result<TradeOffer, TradeError> {
        trade::get_trade_details(env, offer_id)
    }

    /// List active offers
    pub fn list_active_offers(env: Env) -> Result<Vec<BytesN<32>>, TradeError> {
        trade::list_active_offers(env)
    }

    // Barter Agreement Functions
    /// Get barter agreement details
    pub fn get_barter_agreement(env: Env, agreement_id: BytesN<32>) -> Result<BarterAgreement, TradeError> {
        barter::get_barter_agreement(env, agreement_id)
    }

    // Reputation Functions
    /// Update reputation after trade
    pub fn update_reputation(env: Env, cooperative_id: Address, successful: bool) -> Result<(), TradeError> {
        reputation::update_reputation_after_trade(&env, &cooperative_id, successful)
    }

    

}
