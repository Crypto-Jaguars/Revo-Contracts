#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String, Vec};

mod earn;
mod program;
mod redeem;
mod rewards;

#[cfg(test)]
mod tests;

#[contracttype]
pub struct LoyaltyProgram {
    pub program_id: BytesN<32>,
    pub points_per_transaction: u32,
    pub redemption_options: Vec<RedemptionOption>,
}

#[contracttype]
#[derive(Clone)]
pub struct RedemptionOption {
    pub id: u32,
    pub name: String,
    pub points_required: u32,
    pub available_quantity: u32,
}

#[contract]
pub struct LoyaltyContract;

#[contractimpl]
impl LoyaltyContract {
    pub fn create_loyalty_program(
        env: Env,
        program_id: BytesN<32>,
        points_per_transaction: u32,
        redemption_options: Vec<RedemptionOption>,
    ) {
        program::create_loyalty_program(
            &env,
            program_id,
            points_per_transaction,
            redemption_options,
        );
    }

    pub fn award_points(
        env: Env,
        program_id: BytesN<32>,
        user_address: Address,
        transaction_amount: u32,
    ) {
        earn::award_points(&env, program_id, user_address, transaction_amount);
    }

    pub fn redeem_reward(
        env: Env,
        program_id: BytesN<32>,
        user_address: Address,
        redemption_option_id: u32,
    ) {
        redeem::redeem_reward(&env, program_id, user_address, redemption_option_id);
    }

    pub fn get_program_info(env: Env, program_id: BytesN<32>) -> LoyaltyProgram {
        program::get_program_info(&env, program_id)
    }

    pub fn list_available_rewards(env: Env, program_id: BytesN<32>) -> Vec<RedemptionOption> {
        rewards::list_available_rewards(&env, program_id)
    }
}
