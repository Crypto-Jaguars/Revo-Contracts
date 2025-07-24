use crate::error::ContractError;
use crate::storage;
use crate::utils::utils;
use crate::{CertificateManagementContractClient, LoyaltyTokenContractClient};
use soroban_sdk::{Address, BytesN, Env, Symbol};

/// Issues a certificate and rewards to a farmer who has completed a program.
pub fn issue_certificate(
    env: &Env,
    instructor: Address,
    program_id: BytesN<32>,
    farmer_id: Address,
) -> Result<BytesN<32>, ContractError> {
    let mut program = storage::get_program(env, &program_id)?;

    // Verify that the caller is the instructor for this program.
    if program.instructor_id != instructor {
        return Err(ContractError::NotInstructor);
    }

    let mut status = program
        .participants
        .get(farmer_id.clone())
        .ok_or(ContractError::ParticipantNotFound)?;

    // Check if the farmer has completed the program.
    if status.progress < 100 {
        return Err(ContractError::NotCompleted);
    }

    // Check if a certificate has already been issued.
    if status.certificate_id != BytesN::from_array(env, &[0; 32]) {
        return Err(ContractError::AlreadyCertified);
    }

    // --- Issue Certificate via Cross-Contract Call ---
    let certificate_contract_id = storage::get_certificate_contract(env);
    let certificate_client = CertificateManagementContractClient::new(env, &certificate_contract_id);
    
    // Generate a unique ID for this specific certificate. This will serve as the verification hash.
    let certificate_id = utils::generate_id(env, (program_id.clone(), farmer_id.clone()));
    
    // Call the `issue_certification` function on the external certificate contract.
    certificate_client.issue_certification(
        &env.current_contract_address(), // The training contract is the issuer.
        &farmer_id, // The farmer is the recipient.
        &Symbol::new(env, "TrainingCert"), // A generic type for this certificate.
        &0, // Expiration date (0 for non-expiring).
        &certificate_id, // The unique hash for verification.
    );

    // --- Reward Loyalty Points via Cross-Contract Call ---
    let loyalty_token_id = storage::get_loyalty_token(env);
    let loyalty_program_id = storage::get_loyalty_program(env);
    let loyalty_token_client = LoyaltyTokenContractClient::new(env, &loyalty_token_id);
    
    // The `transaction_amount` is set to 1 to represent the completion of one training program.
    // The loyalty contract's `points_per_transaction` will determine the actual points awarded.
    loyalty_token_client.award_points(&loyalty_program_id, &farmer_id, &1);

    // --- Update Participant Status ---
    status.certificate_id = certificate_id.clone();
    program.participants.set(farmer_id, status);
    storage::set_program(env, &program);

    Ok(certificate_id)
}
