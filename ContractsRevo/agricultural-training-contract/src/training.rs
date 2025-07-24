use crate::error::ContractError;
use crate::storage::{self, TrainingProgram};
use crate::utils::utils;
use soroban_sdk::{Address, BytesN, Env, Map, String};

/// Handles the logic for creating and managing training programs.
pub fn create_training_program(
    env: &Env,
    instructor: Address,
    title: String,
    description: String,
    duration_hours: u32,
    materials_hash: BytesN<32>,
) -> Result<BytesN<32>, ContractError> {
    if title.is_empty() || duration_hours == 0 {
        return Err(ContractError::InvalidData);
    }

    // Generate a unique ID for the program.
    let program_id = utils::generate_id(env, (title.clone(), instructor.clone(), env.ledger().timestamp()));

    let program = TrainingProgram {
        program_id: program_id.clone(),
        title,
        description,
        duration_hours,
        instructor_id: instructor,
        materials_hash,
        participants: Map::new(env), // Initialize with an empty map of participants.
    };

    // Save the new program to storage.
    storage::set_program(env, &program);

    Ok(program_id)
}
