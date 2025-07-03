use crate::{LoyaltyProgram, RedemptionOption};
use soroban_sdk::{BytesN, Env, Symbol, Vec};

pub fn create_loyalty_program(
    env: &Env,
    program_id: BytesN<32>,
    points_per_transaction: u32,
    redemption_options: Vec<RedemptionOption>,
) {
    let key = (Symbol::new(env, "program"), program_id.clone());
    if env.storage().persistent().has(&key) {
        panic!("Program already exists");
    }
    let program = LoyaltyProgram {
        program_id,
        points_per_transaction,
        redemption_options,
    };
    env.storage().persistent().set(&key, &program);
}

pub fn get_program_info(env: &Env, program_id: BytesN<32>) -> LoyaltyProgram {
    let key = (Symbol::new(env, "program"), program_id);
    env.storage()
        .persistent()
        .get::<(Symbol, BytesN<32>), LoyaltyProgram>(&key)
        .expect("Program not found")
}
