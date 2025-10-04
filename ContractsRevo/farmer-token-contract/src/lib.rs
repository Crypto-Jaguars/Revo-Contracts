#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

mod burn;
mod mint;
mod token;
mod utils;

pub use burn::*;
pub use mint::*;
pub use token::*;
pub use utils::*;

#[contract]
pub struct FarmerTokenContract;

#[contractimpl]
impl FarmerTokenContract {
    /// Initialize the token contract with admin address and initial metadata
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
    ) -> Result<(), TokenError> {
        token::initialize(env, admin, name, symbol, decimals)
    }

    /// Mint new tokens to a farmer's address
    /// Only authorized addresses (admin or approved minters) can mint
    pub fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), MintError> {
        mint::mint_tokens(env, minter, to, amount)
    }

    /// Burn tokens from an address
    /// Only the token holder or approved burner can burn tokens
    pub fn burn(env: Env, burner: Address, from: Address, amount: i128) -> Result<(), BurnError> {
        burn::burn_tokens(env, burner, from, amount)
    }

    /// Transfer tokens from one address to another
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        token::transfer(env, from, to, amount)
    }

    /// Transfer tokens on behalf of another address (requires approval)
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        token::transfer_from(env, spender, from, to, amount)
    }

    /// Approve an address to spend tokens on behalf of the owner
    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        token::approve(env, owner, spender, amount)
    }

    /// Get the balance of an address
    pub fn balance(env: Env, owner: Address) -> i128 {
        token::balance(env, owner)
    }

    /// Get the allowance of a spender for an owner
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        token::allowance(env, owner, spender)
    }

    /// Get the total supply of tokens
    pub fn total_supply(env: Env) -> i128 {
        token::total_supply(env)
    }

    /// Get token metadata
    pub fn token_metadata(env: Env) -> TokenMetadata {
        token::token_metadata(env)
    }

    /// Add a new minter (admin only)
    pub fn add_minter(env: Env, admin: Address, minter: Address) -> Result<(), AdminError> {
        utils::add_minter(env, admin, minter)
    }

    /// Remove a minter (admin only)
    pub fn remove_minter(env: Env, admin: Address, minter: Address) -> Result<(), AdminError> {
        utils::remove_minter(env, admin, minter)
    }

    /// Check if an address is a minter
    pub fn is_minter(env: Env, address: Address) -> bool {
        utils::is_minter(env, address)
    }

    /// Pause token transfers (admin only)
    pub fn pause(env: Env, admin: Address) -> Result<(), AdminError> {
        utils::pause(env, admin)
    }

    /// Unpause token transfers (admin only)
    pub fn unpause(env: Env, admin: Address) -> Result<(), AdminError> {
        utils::unpause(env, admin)
    }

    /// Check if token transfers are paused
    pub fn is_paused(env: Env) -> bool {
        utils::is_paused(env)
    }

    /// Get the admin address
    pub fn admin(env: Env) -> Result<Address, AdminError> {
        utils::get_admin(env)
    }

    /// Batch mint tokens to multiple farmers
    pub fn batch_mint(
        env: Env,
        minter: Address,
        recipients: soroban_sdk::Vec<(Address, i128)>,
    ) -> Result<(), MintError> {
        mint::batch_mint(env, minter, recipients)
    }

    /// Mint tokens for agricultural milestones
    pub fn mint_for_milestone(
        env: Env,
        minter: Address,
        farmer: Address,
        milestone_type: Symbol,
        amount: i128,
    ) -> Result<(), MintError> {
        mint::mint_for_milestone(env, minter, farmer, milestone_type, amount)
    }

    /// Burn tokens for redemption purposes
    pub fn burn_for_redemption(
        env: Env,
        farmer: Address,
        amount: i128,
        redemption_type: Symbol,
    ) -> Result<(), BurnError> {
        burn::burn_for_redemption(env, farmer, amount, redemption_type)
    }

    /// Burn tokens as a penalty (admin only)
    pub fn burn_as_penalty(
        env: Env,
        admin: Address,
        from: Address,
        amount: i128,
        reason: Symbol,
    ) -> Result<(), BurnError> {
        burn::burn_as_penalty(env, admin, from, amount, reason)
    }
}

#[cfg(test)]
mod test;
