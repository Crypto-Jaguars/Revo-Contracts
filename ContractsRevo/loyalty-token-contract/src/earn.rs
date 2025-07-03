use crate::LoyaltyProgram;
use soroban_sdk::{Address, BytesN, Env, Symbol};

pub fn award_points(
    env: &Env,
    program_id: BytesN<32>,
    user_address: Address,
    transaction_amount: u32,
) {
    let program_key = (Symbol::new(env, "program"), program_id.clone());

    let program: LoyaltyProgram = env
        .storage()
        .persistent()
        .get::<(Symbol, BytesN<32>), LoyaltyProgram>(&program_key)
        .expect("Program not found");

    let points_to_award = (program.points_per_transaction as u64) * (transaction_amount as u64);

    let points_key = (
        Symbol::new(env, "points"),
        program_id.clone(),
        user_address.clone(),
    );

    let current_points: u64 = env
        .storage()
        .persistent()
        .get::<(Symbol, BytesN<32>, Address), u64>(&points_key)
        .unwrap_or(0);

    let new_points = current_points + points_to_award;

    env.storage().persistent().set(&points_key, &new_points);

    env.events().publish(
        (
            Symbol::new(env, "points_awarded"),
            program_id.clone(),
            user_address,
        ),
        points_to_award,
    );
}
