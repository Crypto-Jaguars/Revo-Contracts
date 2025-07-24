use crate::error::ContractError;
use crate::storage::{self, ParticipantStatus};
use soroban_sdk::{Address, BytesN, Env};

/// Enrolls a farmer in a specified training program.
pub fn enroll_farmer(
    env: &Env,
    farmer_id: Address,
    program_id: BytesN<32>,
) -> Result<(), ContractError> {
    let mut program = storage::get_program(env, &program_id)?;

    // Check if the farmer is already enrolled.
    if program.participants.contains_key(farmer_id.clone()) {
        return Err(ContractError::AlreadyEnrolled);
    }

    // Create a new status for the participant.
    let status = ParticipantStatus {
        farmer_id: farmer_id.clone(),
        progress: 0,
        // Initialize certificate_id with a zeroed hash to indicate it's not yet issued.
        certificate_id: BytesN::from_array(env, &[0; 32]),
    };

    // Add the farmer to the program's participant list.
    program.participants.set(farmer_id, status);
    storage::set_program(env, &program);

    Ok(())
}

/// Updates the training progress for a specific farmer.
pub fn update_progress(
    env: &Env,
    instructor: Address,
    program_id: BytesN<32>,
    farmer_id: Address,
    progress_percentage: u32,
) -> Result<(), ContractError> {
    if progress_percentage > 100 {
        return Err(ContractError::InvalidData);
    }

    let mut program = storage::get_program(env, &program_id)?;

    // Verify that the caller is the instructor for this program.
    if program.instructor_id != instructor {
        return Err(ContractError::NotInstructor);
    }

    // Get the current status of the participant.
    let mut status = program
        .participants
        .get(farmer_id.clone())
        .ok_or(ContractError::ParticipantNotFound)?;

    // Update the progress.
    status.progress = progress_percentage;

    // Save the updated status back to the program's participant map.
    program.participants.set(farmer_id, status);
    storage::set_program(env, &program);

    Ok(())
}
