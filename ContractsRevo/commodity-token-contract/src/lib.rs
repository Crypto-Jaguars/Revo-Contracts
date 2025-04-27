#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec, Val};

mod issue;
mod redeem;
mod validate;
mod storage;
mod metadata;

pub use issue::*;
pub use redeem::*;
pub use validate::*;
pub use storage::*;
pub use metadata::*;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommodityBackedToken {
    pub commodity_type: String,
    pub quantity: u32,
    pub grade: String,
    pub storage_location: String,
    pub expiration_date: u64,
    pub verification_data: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inventory {
    pub total_quantity: u32,
    pub available_quantity: u32,
    pub issued_tokens: u32,
}

#[contract]
pub struct CommodityTokenContract;

#[contractimpl]
impl CommodityTokenContract {
    // Initialize the contract with admin address
    pub fn initialize(env: Env, admin: Address) {
        // Save admin address
        storage::set_admin(&env, &admin);
    }
    
    // Issue new tokens
    pub fn issue_token(
        env: Env,
        issuer: Address,
        commodity_type: String,
        quantity: u32,
        grade: String,
        storage_location: String,
        expiration_date: u64,
        verification_data: BytesN<32>,
    ) -> BytesN<32> {
        issuer.require_auth();
        issue::issue_token(
            &env, 
            &issuer, 
            &commodity_type, 
            quantity, 
            &grade, 
            &storage_location, 
            expiration_date,
            &verification_data,
        )
    }
    
    // Redeem tokens for physical commodities
    pub fn redeem_token(
        env: Env,
        token_id: BytesN<32>,
        redeemer: Address,
        quantity: u32,
    ) {
        redeemer.require_auth();
        redeem::redeem_token(&env, &token_id, &redeemer, quantity);
    }
    
    // Get token metadata
    pub fn get_token_metadata(env: Env, token_id: BytesN<32>) -> CommodityBackedToken {
        metadata::get_token_metadata(&env, &token_id)
    }
    
    // List available inventory
    pub fn list_available_inventory(env: Env, commodity_type: String) -> Inventory {
        storage::get_inventory(&env, &commodity_type)
    }
    
    // Validate commodity data
    pub fn validate_commodity(
        env: Env,
        commodity_type: String,
        verification_data: BytesN<32>
    ) -> bool {
        validate::validate_commodity(&env, &commodity_type, &verification_data)
    }
    
    // Add inventory (admin only)
    pub fn add_inventory(
        env: Env,
        admin: Address,
        commodity_type: String,
        quantity: u32,
    ) {
        admin.require_auth();
        storage::add_inventory(&env, &admin, &commodity_type, quantity);
    }
    
    // Register commodity verification data (admin only)
    pub fn register_commodity_verification(
        env: Env,
        admin: Address,
        commodity_type: String,
        verification_data: BytesN<32>,
        metadata: Map<String, String>,
    ) {
        admin.require_auth();
        validate::register_commodity_verification(&env, &admin, &commodity_type, &verification_data, &metadata);
    }
    
    // Add authorized issuer (admin only)
    pub fn add_authorized_issuer(
        env: Env,
        admin: Address,
        issuer: Address,
    ) {
        admin.require_auth();
        storage::add_authorized_issuer(&env, &admin, &issuer);
    }
    
    // List tokens by commodity type
    pub fn list_tokens_by_commodity(
        env: Env,
        commodity_type: String,
    ) -> Vec<BytesN<32>> {
        metadata::list_tokens_by_commodity(&env, &commodity_type)
    }
    
    // Get token details (human-readable)
    pub fn get_token_details(
        env: Env,
        token_id: BytesN<32>,
    ) -> Map<String, Val> {
        metadata::get_token_details(&env, &token_id)
    }
}