#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Val, Vec,
};

mod error;
mod issue;
mod metadata;
mod redeem;
mod storage;
mod validate;

pub use error::*;
pub use issue::*;
pub use metadata::*;
pub use redeem::*;
pub use storage::*;
pub use validate::*;

#[cfg(test)]
mod test;
#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

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
#[derive(Clone)]
pub struct CommodityTokenContract;

#[contractimpl]
impl CommodityTokenContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&storage::DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        Ok(())
    }

    pub fn issue_token(
        env: Env,
        issuer: Address,
        commodity_type: String,
        quantity: u32,
        grade: String,
        storage_location: String,
        expiration_date: u64,
        verification_data: BytesN<32>,
    ) -> Result<BytesN<32>, IssueError> {
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

    pub fn redeem_token(
        env: Env,
        token_id: BytesN<32>,
        redeemer: Address,
        quantity: u32,
    ) -> Result<(), RedeemError> {
        redeemer.require_auth();
        redeem::redeem_token(&env, &token_id, &redeemer, quantity)
    }

    pub fn get_token_metadata(
        env: Env,
        token_id: BytesN<32>,
    ) -> Result<CommodityBackedToken, ContractError> {
        metadata::get_token_metadata(&env, &token_id)
    }

    pub fn list_available_inventory(env: Env, commodity_type: String) -> Inventory {
        storage::get_inventory(&env, &commodity_type)
    }

    pub fn validate_commodity(
        env: Env,
        commodity_type: String,
        verification_data: BytesN<32>,
    ) -> bool {
        validate::validate_commodity(&env, &commodity_type, &verification_data)
    }

    pub fn add_inventory(
        env: Env,
        admin: Address,
        commodity_type: String,
        quantity: u32,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        storage::add_inventory(&env, &admin, &commodity_type, quantity)
    }

    pub fn register_commodity_verification(
        env: Env,
        admin: Address,
        commodity_type: String,
        verification_data: BytesN<32>,
        metadata: Map<String, String>,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        validate::register_commodity_verification(
            &env,
            &admin,
            &commodity_type,
            &verification_data,
            &metadata,
        )
    }

    pub fn add_authorized_issuer(
        env: Env,
        admin: Address,
        issuer: Address,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        storage::add_authorized_issuer(&env, &admin, &issuer)
    }

    pub fn list_tokens_by_commodity(env: Env, commodity_type: String) -> Vec<BytesN<32>> {
        metadata::list_tokens_by_commodity(&env, &commodity_type)
    }

    pub fn get_token_details(
        env: Env,
        token_id: BytesN<32>,
    ) -> Result<Map<String, Val>, ContractError> {
        metadata::get_token_details(&env, &token_id)
    }
}
