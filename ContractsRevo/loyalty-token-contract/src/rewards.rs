use crate::{program::get_program_info, RedemptionOption};
use soroban_sdk::{BytesN, Env, Vec};

pub fn list_available_rewards(env: &Env, program_id: BytesN<32>) -> Vec<RedemptionOption> {
    let program = get_program_info(env, program_id);

    let mut available_rewards = Vec::new(env);

    for opt in program.redemption_options {
        if opt.available_quantity > 0 {
            available_rewards.push_back(opt);
        }
    }
    available_rewards
}
