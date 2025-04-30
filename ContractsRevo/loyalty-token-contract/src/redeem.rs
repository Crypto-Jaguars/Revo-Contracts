use soroban_sdk::{Env, BytesN, Address, Symbol};
use crate::LoyaltyProgram;

pub fn redeem_reward(env: &Env, program_id: BytesN<32>, user_address: Address, redemption_option_id: u32) {
    let program_key = (Symbol::new(env, "program"), program_id.clone());
    let mut program: LoyaltyProgram = env.storage()
        .persistent()
        .get::<(Symbol, BytesN<32>), LoyaltyProgram>(&program_key)
        .expect("Program not found");
    let points_key = (Symbol::new(env, "points"), program_id.clone(), user_address.clone());
    let user_points: u64 = env.storage()
        .persistent()
        .get::<(Symbol, BytesN<32>, Address), u64>(&points_key)
        .unwrap_or(0);
    
    let option_index = program.redemption_options.iter().position(|opt| opt.id == redemption_option_id).expect("Redemption option not found");
    let mut option = program.redemption_options.get(option_index.try_into().unwrap()).expect("Redemption option not found");
    
    if option.available_quantity == 0 {
        panic!("Reward is out of stock");
    }
    if user_points < option.points_required as u64 {
        panic!("Insufficient points");
    }
    
    let new_points = user_points - option.points_required as u64;
    env.storage().persistent().set(&points_key, &new_points);
    
    option.available_quantity -= 1;
    program.redemption_options.set(option_index.try_into().unwrap(), option);
    
    env.storage().persistent().set(&program_key, &program);
    
    env.events().publish(
        (Symbol::new(env, "reward_redeemed"), program_id, user_address),
        redemption_option_id
    );
}