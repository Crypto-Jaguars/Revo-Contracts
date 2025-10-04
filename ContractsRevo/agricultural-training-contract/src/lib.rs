#![no_std]

mod certification;
mod error;
mod participation;
mod storage;
mod test;
mod training;
mod utils;

pub use error::ContractError;
pub use storage::{ParticipantStatus, TrainingProgram};

use soroban_sdk::{contract, contractclient, contractimpl, Address, BytesN, Env, String, Symbol};

// Manually define the interface for the external certificate management contract.
#[contractclient(name = "CertificateManagementContractClient")]
pub trait CertificateManagementContract {
    fn issue_certification(
        env: Env,
        issuer: Address,
        recipient: Address,
        cert_type: Symbol,
        expiration_date: u64,
        verification_hash: BytesN<32>,
    );
}

// Manually define the interface for the external loyalty token contract.
#[contractclient(name = "LoyaltyTokenContractClient")]
pub trait LoyaltyTokenContract {
    fn award_points(
        env: Env,
        program_id: BytesN<32>,
        user_address: Address,
        transaction_amount: u32,
    );
}

#[contract]
pub struct AgriculturalTrainingContract;

#[contractimpl]
impl AgriculturalTrainingContract {
    /// Initializes the contract with an admin and the addresses of external contracts.
    pub fn initialize(
        env: Env,
        admin: Address,
        certificate_contract_id: Address,
        loyalty_token_id: Address,
        loyalty_program_id: BytesN<32>,
    ) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_certificate_contract(&env, &certificate_contract_id);
        storage::set_loyalty_token(&env, &loyalty_token_id);
        storage::set_loyalty_program(&env, &loyalty_program_id);
        Ok(())
    }

    /// Creates a new training program.
    pub fn create_training_program(
        env: Env,
        instructor: Address,
        title: String,
        description: String,
        duration_hours: u32,
        materials_hash: BytesN<32>,
    ) -> Result<BytesN<32>, ContractError> {
        instructor.require_auth();
        training::create_training_program(
            &env,
            instructor,
            title,
            description,
            duration_hours,
            materials_hash,
        )
    }

    /// Enrolls a farmer in a specific training program.
    pub fn enroll_farmer(
        env: Env,
        farmer: Address,
        program_id: BytesN<32>,
    ) -> Result<(), ContractError> {
        farmer.require_auth();
        participation::enroll_farmer(&env, farmer, program_id)
    }

    /// Updates the progress of a farmer in a training program.
    pub fn update_progress(
        env: Env,
        instructor: Address,
        program_id: BytesN<32>,
        farmer_id: Address,
        progress_percentage: u32,
    ) -> Result<(), ContractError> {
        instructor.require_auth();
        participation::update_progress(&env, instructor, program_id, farmer_id, progress_percentage)
    }

    /// Issues a tokenized certificate and rewards loyalty points upon completion.
    pub fn issue_certificate(
        env: Env,
        instructor: Address,
        program_id: BytesN<32>,
        farmer_id: Address,
    ) -> Result<BytesN<32>, ContractError> {
        instructor.require_auth();
        certification::issue_certificate(&env, instructor, program_id, farmer_id)
    }

    // --- Read-Only Functions ---

    /// Retrieves the details of a specific training program.
    pub fn get_program(env: Env, program_id: BytesN<32>) -> Result<TrainingProgram, ContractError> {
        storage::get_program(&env, &program_id)
    }

    /// Retrieves the participation status of a specific farmer in a program.
    pub fn get_participant_status(
        env: Env,
        program_id: BytesN<32>,
        farmer_id: Address,
    ) -> Result<ParticipantStatus, ContractError> {
        storage::get_participant_status(&env, &program_id, &farmer_id)
    }
}
