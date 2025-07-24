use soroban_sdk::{contracttype, Address, BytesN, Env, Map, String};
use crate::error::ContractError;

// --- Data Structures ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantStatus {
    pub farmer_id: Address,
    pub progress: u32, // Percentage completed (0-100)
    pub certificate_id: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrainingProgram {
    pub program_id: BytesN<32>,
    pub title: String,
    pub description: String,
    pub duration_hours: u32,
    pub instructor_id: Address,
    pub materials_hash: BytesN<32>,
    pub participants: Map<Address, ParticipantStatus>,
}

// --- Storage Keys ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Admin,
    CertificateContract,
    LoyaltyToken,
    LoyaltyProgram,
    Program(BytesN<32>),
}

// --- Admin and Token Management ---

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&StorageKey::Admin, admin);
}

pub fn set_certificate_contract(env: &Env, contract_id: &Address) {
    env.storage().instance().set(&StorageKey::CertificateContract, contract_id);
}

pub fn get_certificate_contract(env: &Env) -> Address {
    env.storage().instance().get(&StorageKey::CertificateContract).unwrap()
}

pub fn set_loyalty_token(env: &Env, token_id: &Address) {
    env.storage().instance().set(&StorageKey::LoyaltyToken, token_id);
}

pub fn get_loyalty_token(env: &Env) -> Address {
    env.storage().instance().get(&StorageKey::LoyaltyToken).unwrap()
}

pub fn set_loyalty_program(env: &Env, program_id: &BytesN<32>) {
    env.storage().instance().set(&StorageKey::LoyaltyProgram, program_id);
}

pub fn get_loyalty_program(env: &Env) -> BytesN<32> {
    env.storage().instance().get(&StorageKey::LoyaltyProgram).unwrap()
}


// --- Program Management ---

pub fn get_program(env: &Env, program_id: &BytesN<32>) -> Result<TrainingProgram, ContractError> {
    env.storage()
        .persistent()
        .get(&StorageKey::Program(program_id.clone()))
        .ok_or(ContractError::ProgramNotFound)
}

pub fn set_program(env: &Env, program: &TrainingProgram) {
    env.storage()
        .persistent()
        .set(&StorageKey::Program(program.program_id.clone()), program);
}

// --- Participant Status ---

pub fn get_participant_status(
    env: &Env,
    program_id: &BytesN<32>,
    farmer_id: &Address,
) -> Result<ParticipantStatus, ContractError> {
    let program = get_program(env, program_id)?;
    program
        .participants
        .get(farmer_id.clone())
        .ok_or(ContractError::ParticipantNotFound)
}
